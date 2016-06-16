
pub enum NetworkFailure {
	TimedOut,
	Bind,
	SocketWrite,
	SocketRead,
	SocketError,
	UnsupportedAction,
}