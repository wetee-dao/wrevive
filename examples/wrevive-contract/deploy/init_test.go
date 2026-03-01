package contracts

import (
	"crypto/rand"
	"errors"
	"fmt"
	"math/big"
	"os"
	"testing"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	chain "github.com/wetee-dao/ink.go"
	"github.com/wetee-dao/ink.go/util"
)

func TestDeployContract(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	/// init pod
	podData, err := os.ReadFile("../../../target/wrevive-example.release.polkavm")
	if err != nil {
		util.LogWithPurple("read file error", err)
		panic(err)
	}

	// fmt.Println("podCode", podCode)
	salt := genSalt()
	address, err := client.DeployContract(
		util.InkCode{Upload: &podData},
		&pk, types.NewU128(*big.NewInt(0)),
		util.InkContractInput{
			Selector: "0x00000000",
			Args:     []any{},
		},
		util.NewSome(salt),
	)

	if err != nil {
		util.LogWithPurple("DeployContract", err)
		panic(err)
	}
	fmt.Println("address", address.Hex())

	value, _, err := QueryFunction(*address, client, chain.DefaultParamWithOrigin(pk.AccountID()))
	if err != nil {
		util.LogWithPurple("QueryFunction", err)
		panic(err)
	}
	fmt.Println("value", value)
}

type Contract struct {
	ChainClient *chain.ChainClient
	Address     types.H160
}

func (c *Contract) Client() *chain.ChainClient {
	return c.ChainClient
}

func (c *Contract) ContractAddress() types.H160 {
	return c.Address
}

func QueryFunction(
	address types.H160,
	client *chain.ChainClient,
	__ink_params chain.DryRunParams,
) (*uint32, *chain.DryRunReturnGas, error) {
	v, gas, err := chain.DryRunInk[uint32](
		&Contract{
			ChainClient: client,
			Address:     address,
		},
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xfac42ee4",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func genSalt() [32]byte {
	bytes := make([]byte, 32)
	_, err := rand.Read(bytes)
	if err != nil {
		panic(err)
	}
	randomBytes := [32]byte{}
	copy(randomBytes[:], bytes)

	return randomBytes
}
