package contracts

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"os"
	"testing"
	"wetee/test/wrevive_example"

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

	// upload pod code
	codeHash, err := client.UploadInkCode(podData, &pk)
	if err != nil {
		util.LogWithPurple("UploadInkCode", err)
		panic(err)
	}

	// fmt.Println("podCode", podCode)
	salt := genSalt()
	address, err := wrevive_example.DeployWreviveExampleWithDeploy(0, chain.DeployParams{
		Client: client,
		Signer: &pk,
		Code:   util.InkCode{Existing: codeHash},
		Salt:   util.NewSome(salt),
	})
	if err != nil {
		util.LogWithPurple("DeployWreviveExampleWithDeploy", err)
		panic(err)
	}
	fmt.Println("address", address.Hex())

	contract, err := wrevive_example.InitWreviveExampleContract(client, address.Hex())
	if err != nil {
		util.LogWithPurple("InitWreviveExampleContract", err)
		panic(err)
	}
	fmt.Println("contract", contract)

	err = contract.ExecSetValue(100, chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		util.LogWithPurple("ExecSetValue", err)
		panic(err)
	}

	value, _, err := contract.QueryGetValue(chain.DefaultParamWithOrigin(pk.AccountID()))
	if err != nil {
		util.LogWithPurple("QueryGetValue", err)
		panic(err)
	}
	fmt.Println("value", *value)

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
