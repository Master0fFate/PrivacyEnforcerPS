using PrivacyEnforcerPro.Core.Enums;

namespace PrivacyEnforcerPro.Core.Models;

public sealed class OperationResult
{
    public OperationStatus Status { get; init; }
    public string Message { get; init; } = string.Empty;
    public string? OperationId { get; init; }

    public static OperationResult Success(string message = "", string? id = null) => new()
    {
        Status = OperationStatus.Success,
        Message = message,
        OperationId = id
    };

    public static OperationResult Failure(string message, string? id = null) => new()
    {
        Status = OperationStatus.Failure,
        Message = message,
        OperationId = id
    };

    public static OperationResult Warning(string message, string? id = null) => new()
    {
        Status = OperationStatus.Warning,
        Message = message,
        OperationId = id
    };
}