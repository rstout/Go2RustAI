type Reader interface {
	AccountReader
	Balance(addr solana.PublicKey) (uint64, error)
	SlotHeight() (uint64, error)
	LatestBlockhash() (*rpc.GetLatestBlockhashResult, error)
	ChainID() (string, error)
	GetFeeForMessage(msg string) (uint64, error)
	GetLatestBlock() (*rpc.GetBlockResult, error)
}