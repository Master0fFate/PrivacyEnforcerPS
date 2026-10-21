namespace PrivacyEnforcerPro.Core.Policies;

// No platform, registry, filesystem, process or network access: all side effects are injected.
public sealed class PrivacyEngine(IPolicyStore store)
{
    public IReadOnlyList<AuditFinding> Audit(IEnumerable<PrivacyPolicy> policies) => policies.Select(p =>
    {
        try
        {
            var value = store.Read(p);
            return new AuditFinding(p.Id, value.Exists && value.Value == p.Desired ? "Configured" : "DifferentOrNotConfigured", value, p.Desired, p.Impact);
        }
        catch (Exception ex) { return new AuditFinding(p.Id, "Unknown", null, p.Desired, p.Impact, ex.Message); }
    }).ToArray();

    public PrivacyBackup Plan(IEnumerable<PrivacyPolicy> policies, string machine)
    {
        // Read everything before writing anything. Unexpected registry types fail closed.
        var changes = policies.Select(p => new PolicyChange(p.Id, store.Read(p), p.Desired))
            .Where(c => !c.Before.Exists || c.Before.Value != c.Desired).ToList();
        var backup = new PrivacyBackup(1, machine, DateTimeOffset.UtcNow, changes);
        Validate(backup, machine);
        return backup;
    }

    public void Apply(PrivacyBackup plan, string machine, Action<PrivacyBackup> persistBackup)
    {
        Validate(plan, machine);
        if (plan.Changes.Count == 0) return;
        EnsureUnchanged(plan.Changes, restore: false);
        persistBackup(plan); // Must succeed durably before the first registry write.
        foreach (var change in plan.Changes)
        {
            var policy = PolicyCatalog.Get(change.Id);
            if (store.Read(policy) != change.Before) throw new InvalidOperationException($"{change.Id} changed during apply. Stop and audit; use the saved backup to recover earlier writes.");
            var desired = new PolicyValue(true, change.Desired);
            store.Write(policy, desired);
            if (store.Read(policy) != desired) throw new IOException($"Read-back failed for {change.Id}. Use the saved backup to recover.");
        }
    }

    public void Restore(PrivacyBackup backup, string machine, bool dryRun)
    {
        Validate(backup, machine);
        EnsureUnchanged(backup.Changes, restore: true);
        if (dryRun) return;
        foreach (var change in backup.Changes.AsEnumerable().Reverse())
        {
            var policy = PolicyCatalog.Get(change.Id);
            var current = store.Read(policy);
            if (current == change.Before) continue; // Supports partial apply and repeated restore.
            if (current != new PolicyValue(true, change.Desired)) throw new InvalidOperationException($"{change.Id} changed during restore. Stop and audit.");
            store.Write(policy, change.Before);
            if (store.Read(policy) != change.Before) throw new IOException($"Read-back failed restoring {change.Id}; retain the backup.");
        }
    }

    private void EnsureUnchanged(IEnumerable<PolicyChange> changes, bool restore)
    {
        foreach (var change in changes)
        {
            var current = store.Read(PolicyCatalog.Get(change.Id));
            if (current != change.Before && (!restore || current != new PolicyValue(true, change.Desired)))
                throw new InvalidOperationException($"Conflict for {change.Id}: current value changed outside this plan. No automatic overwrite; audit first.");
        }
    }

    public static void Validate(PrivacyBackup backup, string machine)
    {
        if (backup is null || backup.Version != 1 || backup.Machine != machine || backup.Changes is null || backup.Changes.Count > PolicyCatalog.All.Count)
            throw new ArgumentException("Invalid backup version, machine identity or change count.");
        var ids = new HashSet<string>(StringComparer.Ordinal);
        foreach (var change in backup.Changes)
        {
            if (change is null || change.Before is null || !ids.Add(change.Id)) throw new ArgumentException("Invalid or duplicate backup entry.");
            var policy = PolicyCatalog.Get(change.Id);
            if (change.Desired != policy.Desired || (!change.Before.Exists && change.Before.Value != 0))
                throw new ArgumentException("Backup entry does not match the policy catalog.");
        }
    }
}
