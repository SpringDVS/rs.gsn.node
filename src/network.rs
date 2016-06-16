
pub enum NetworkFailure {
	TimedOut,
	Bind,
	SocketWrite,
	SocketRead,
	UnsupportedAction,
}