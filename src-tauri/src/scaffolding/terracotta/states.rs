enum TerracottaState {
    Idle,
    HostScanning,
    HostStarting,
    HostOk,
    GuestConnecting,
    GuestStarting,
    GuestOk,
    Exception(ExceptionType),
}

enum ExceptionType {
    PingHostFail,
    PingHostRst,
    GuestEasytierCrash,
    HostEasytierCrash,
    PingServerRst,
    ScaffoldingInvalidResponse,
}