using PrivacyEnforcerPro.Core.Policies;
using PrivacyEnforcerPro.Infrastructure.Services;

namespace PrivacyEnforcerPro.Tests;

public sealed class PrivacyEngineTests
{
    private const string Machine = "test-machine";
    private sealed class MemoryStore : IPolicyStore
    {
        public Dictionary<string, PolicyValue> Values { get; } = new();
        public int Writes { get; private set; }
        public string? FailWrite { get; set; }
        public string? FailRead { get; set; }
        public bool DropWrites { get; set; }
        public PolicyValue Read(PrivacyPolicy p) => p.Id == FailRead ? throw new IOException("Denied") : Values.GetValueOrDefault(p.Id, new(false));
        public void Write(PrivacyPolicy p, PolicyValue v)
        {
            if (p.Id == FailWrite) throw new IOException("Injected failure");
            Writes++;
            if (!DropWrites) Values[p.Id] = v;
        }
    }
    [Fact] public void AuditAndPlanNeverWrite()
    {
        var store = new MemoryStore(); var engine = new PrivacyEngine(store);
        Assert.All(engine.Audit(PolicyCatalog.All), f => Assert.Equal("DifferentOrNotConfigured", f.State));
        Assert.Equal(7, engine.Plan(PolicyCatalog.All, Machine).Changes.Count);
        Assert.Equal(0, store.Writes);
    }
    [Fact] public void AuditReportsReadFailureAsUnknown()
    {
        var store = new MemoryStore { FailRead = "advertising-id" };
        Assert.Equal("Unknown", new PrivacyEngine(store).Audit(PolicyCatalog.All)[0].State);
        Assert.Throws<IOException>(() => new PrivacyEngine(store).Plan(PolicyCatalog.All, Machine));
        Assert.Equal(0, store.Writes);
    }
    [Fact] public void BackupFailurePreventsAllWrites()
    {
        var store = new MemoryStore(); var engine = new PrivacyEngine(store);
        Assert.Throws<IOException>(() => engine.Apply(engine.Plan(PolicyCatalog.All, Machine), Machine, _ => throw new IOException()));
        Assert.Equal(0, store.Writes);
    }
    [Fact] public void BackupPrecedesWritesAndApplyIsIdempotent()
    {
        var store = new MemoryStore(); var engine = new PrivacyEngine(store);
        engine.Apply(engine.Plan(PolicyCatalog.All, Machine), Machine, _ => Assert.Equal(0, store.Writes));
        Assert.All(engine.Audit(PolicyCatalog.All), f => Assert.Equal("Configured", f.State));
        engine.Apply(engine.Plan(PolicyCatalog.All, Machine), Machine, _ => throw new Exception("Should not need another backup"));
        Assert.Equal(7, store.Writes);
    }
    [Fact] public void RestorePreservesAbsentAndExplicitZeroValues()
    {
        var store = new MemoryStore(); store.Values["advertising-id"] = new(true, 0);
        var engine = new PrivacyEngine(store); var backup = engine.Plan(PolicyCatalog.All, Machine);
        engine.Apply(backup, Machine, _ => { }); engine.Restore(backup, Machine, true);
        Assert.Equal(7, store.Writes);
        engine.Restore(backup, Machine, false);
        Assert.Equal(new(true, 0), store.Values["advertising-id"]);
        Assert.False(store.Values["clipboard-sync"].Exists);
        engine.Restore(backup, Machine, false); Assert.Equal(14, store.Writes);
    }
    [Fact] public void PartialFailureCanBeRestored()
    {
        var store = new MemoryStore { FailWrite = "activity-feed" }; var engine = new PrivacyEngine(store);
        var backup = engine.Plan(PolicyCatalog.All, Machine);
        Assert.Throws<IOException>(() => engine.Apply(backup, Machine, _ => { }));
        store.FailWrite = null; engine.Restore(backup, Machine, false);
        Assert.All(PolicyCatalog.All, p => Assert.False(store.Read(p).Exists));
    }
    [Fact] public void ExternalChangesBlockRestoreBeforeAnyWrite()
    {
        var store = new MemoryStore(); var engine = new PrivacyEngine(store);
        var backup = engine.Plan(PolicyCatalog.All, Machine); engine.Apply(backup, Machine, _ => { });
        store.Values["app-location"] = new(true, 123);
        Assert.Throws<InvalidOperationException>(() => engine.Restore(backup, Machine, false));
        Assert.Equal(7, store.Writes);
    }
    [Fact] public void StaleApplyPlanIsRejectedBeforeBackup()
    {
        var store = new MemoryStore(); var engine = new PrivacyEngine(store);
        var plan = engine.Plan(PolicyCatalog.All, Machine); store.Values["app-location"] = new(true, 123);
        Assert.Throws<InvalidOperationException>(() => engine.Apply(plan, Machine, _ => throw new Exception("must not save")));
        Assert.Equal(0, store.Writes);
    }
    [Fact] public void ReadBackFailureIsNotReportedAsSuccess()
    {
        var store = new MemoryStore { DropWrites = true }; var engine = new PrivacyEngine(store);
        Assert.Throws<IOException>(() => engine.Apply(engine.Plan(PolicyCatalog.All, Machine), Machine, _ => { }));
        Assert.Equal(1, store.Writes);
    }
    [Fact] public void MalformedOrForeignBackupsAreRejected()
    {
        var valid = new PrivacyEngine(new MemoryStore()).Plan(PolicyCatalog.All, Machine);
        Assert.Throws<ArgumentException>(() => PrivacyEngine.Validate(valid with { Version = 99 }, Machine));
        Assert.Throws<ArgumentException>(() => PrivacyEngine.Validate(valid, "different"));
        Assert.Throws<ArgumentException>(() => PrivacyEngine.Validate(valid with { Changes = [valid.Changes[0], valid.Changes[0]] }, Machine));
        Assert.Throws<ArgumentException>(() => PrivacyEngine.Validate(valid with { Changes = [new("unknown", new(false), 1)] }, Machine));
        Assert.Throws<ArgumentException>(() => PrivacyEngine.Validate(valid with { Changes = [valid.Changes[0] with { Desired = 42 }] }, Machine));
    }
    [Fact] public void BackupRoundTripsAndCannotOverwrite()
    {
        var path = Path.Combine(Path.GetTempPath(), Guid.NewGuid() + ".json");
        try
        {
            var plan = new PrivacyEngine(new MemoryStore()).Plan(PolicyCatalog.All, Machine);
            PrivacyBackupFile.Save(path, plan);
            var loaded = PrivacyBackupFile.Load(path);
            Assert.Equal(plan.Changes, loaded.Changes);
            Assert.Throws<IOException>(() => PrivacyBackupFile.Save(path, plan));
            File.WriteAllText(path, "{\"unknown\":42}");
            Assert.Throws<System.Text.Json.JsonException>(() => PrivacyBackupFile.Load(path));
        }
        finally { File.Delete(path); }
    }
    [Theory]
    [InlineData("{}")]
    [InlineData("{\"Exists\":true}")]
    [InlineData("{\"Value\":0}")]
    [InlineData("{\"Exists\":true,\"Value\":1,\"Value\":0}")]
    public void IncompleteOrDuplicateRecoveryFieldsAreRejected(string before)
    {
        var path = Path.GetTempFileName();
        var store = new MemoryStore();
        try
        {
            File.WriteAllText(path, "{\"Version\":1,\"Machine\":\"test-machine\",\"Created\":\"2026-09-30T00:00:00Z\",\"Changes\":[{\"Id\":\"clipboard-sync\",\"Desired\":0,\"Before\":" + before + "}]}");
            Assert.Throws<System.Text.Json.JsonException>(() => new PrivacyEngine(store).Restore(PrivacyBackupFile.Load(path), Machine, false));
            Assert.Equal(0, store.Writes);
        }
        finally { File.Delete(path); }
    }
    [Fact] public void OversizedBackupIsRejected()
    {
        var path = Path.GetTempFileName();
        try { File.WriteAllText(path, new string(' ', 65537)); Assert.Throws<ArgumentException>(() => PrivacyBackupFile.Load(path)); }
        finally { File.Delete(path); }
    }
    [Theory]
    [InlineData("enforce")]
    [InlineData("enforce --select unknown")]
    [InlineData("enforce --select advertising-id --apply")]
    [InlineData("audit --apply")]
    [InlineData("restore")]
    [InlineData("restore --backup a --select advertising-id")]
    [InlineData("enforce --select advertising-id --json")]
    [InlineData("audit --surprise")]
    [InlineData("audit --select")]
    [InlineData("list --json --json")]
    public void InvalidCommandsFailClosed(string args) => Assert.Throws<ArgumentException>(() => CommandOptions.Parse(args.Split(' ')));
    [Fact] public void NoArgumentsMeansAudit() => Assert.Equal("audit", CommandOptions.Parse([]).Command);
    [Fact] public void EnforceDefaultsToDryRun() => Assert.False(CommandOptions.Parse(["enforce", "--select", "advertising-id"]).Apply);
    [Fact] public void ExplicitSelectionDeduplicates() => Assert.Single(PolicyCatalog.Select(["advertising-id", "advertising-id"]));
}
