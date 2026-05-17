package wrevive_example

import (
	"errors"
	"fmt"
	"math/big"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	chain "github.com/wetee-dao/ink.go"
	"github.com/wetee-dao/ink.go/util"
)

func DeployWreviveExampleWithDeploy(initial_value uint32, __ink_params chain.DeployParams) (*types.H160, error) {
	return __ink_params.Client.DeployContract(
		__ink_params.Code, __ink_params.Signer, types.NewU128(*big.NewInt(0)),
		util.InkContractInput{
			Selector: "0x00000000",
			Args:     []any{initial_value},
		},
		__ink_params.Salt,
	)
}

func InitWreviveExampleContract(client *chain.ChainClient, address string) (*WreviveExample, error) {
	contractAddress, err := util.HexToH160(address)
	if err != nil {
		return nil, err
	}
	return &WreviveExample{
		ChainClient: client,
		Address:     contractAddress,
	}, nil
}

type WreviveExample struct {
	ChainClient *chain.ChainClient
	Address     types.H160
}

func (c *WreviveExample) Client() *chain.ChainClient {
	return c.ChainClient
}

func (c *WreviveExample) ContractAddress() types.H160 {
	return c.Address
}

func (c *WreviveExample) DryRunSetValue(
	value uint32, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "set_value")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xafd79056",
			Args:     []any{value},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *WreviveExample) ExecSetValue(
	value uint32, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSetValue(value, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xafd79056",
			Args:     []any{value},
		},
		__ink_params,
	)
}

func (c *WreviveExample) CallOfSetValue(
	value uint32, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSetValue(value, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xafd79056",
			Args:     []any{value},
		},
	)
}

func (c *WreviveExample) QueryGetValue(
	__ink_params chain.DryRunParams,
) (*uint32, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "get_value")
	}
	v, gas, err := chain.DryRunInk[uint32](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xc26813d3",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) QueryGetValueOption(
	__ink_params chain.DryRunParams,
) (*util.Option[uint32], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "get_value_option")
	}
	v, gas, err := chain.DryRunInk[util.Option[uint32]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x85e5e3bd",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) DryRunSetValueSol(
	value uint32, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "set_value_sol")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xfac67fb1",
			Args:     []any{value},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *WreviveExample) ExecSetValueSol(
	value uint32, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSetValueSol(value, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xfac67fb1",
			Args:     []any{value},
		},
		__ink_params,
	)
}

func (c *WreviveExample) CallOfSetValueSol(
	value uint32, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSetValueSol(value, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xfac67fb1",
			Args:     []any{value},
		},
	)
}

func (c *WreviveExample) QueryGetValueSol(
	__ink_params chain.DryRunParams,
) (*uint32, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "get_value_sol")
	}
	v, gas, err := chain.DryRunInk[uint32](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x6e2dfdb3",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) QueryGetCluster(
	__ink_params chain.DryRunParams,
) (*Cluster, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "get_cluster")
	}
	v, gas, err := chain.DryRunInk[Cluster](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x85106573",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) DryRunSetCluster(
	cluster Cluster, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "set_cluster")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xc09c2406",
			Args:     []any{cluster},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *WreviveExample) ExecSetCluster(
	cluster Cluster, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSetCluster(cluster, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xc09c2406",
			Args:     []any{cluster},
		},
		__ink_params,
	)
}

func (c *WreviveExample) CallOfSetCluster(
	cluster Cluster, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSetCluster(cluster, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xc09c2406",
			Args:     []any{cluster},
		},
	)
}

func (c *WreviveExample) DryRunSetOwner(
	new_owner types.H160, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "set_owner")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x553f734f",
			Args:     []any{new_owner},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *WreviveExample) ExecSetOwner(
	new_owner types.H160, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSetOwner(new_owner, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x553f734f",
			Args:     []any{new_owner},
		},
		__ink_params,
	)
}

func (c *WreviveExample) CallOfSetOwner(
	new_owner types.H160, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSetOwner(new_owner, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x553f734f",
			Args:     []any{new_owner},
		},
	)
}

func (c *WreviveExample) QueryGetOwner(
	__ink_params chain.DryRunParams,
) (*types.H160, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "get_owner")
	}
	v, gas, err := chain.DryRunInk[types.H160](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xabee0bfa",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) DryRunSetBalance(
	user types.H160, balance uint64, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "set_balance")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x8ba08f48",
			Args:     []any{user, balance},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *WreviveExample) ExecSetBalance(
	user types.H160, balance uint64, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSetBalance(user, balance, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x8ba08f48",
			Args:     []any{user, balance},
		},
		__ink_params,
	)
}

func (c *WreviveExample) CallOfSetBalance(
	user types.H160, balance uint64, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSetBalance(user, balance, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x8ba08f48",
			Args:     []any{user, balance},
		},
	)
}

func (c *WreviveExample) QueryGetBalance(
	user types.H160, __ink_params chain.DryRunParams,
) (*uint64, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "get_balance")
	}
	v, gas, err := chain.DryRunInk[uint64](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x3b9e11d4",
			Args:     []any{user},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) DryRunSetUserInfo(
	user types.H160, info_type byte, value uint32, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "set_user_info")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x1adb3149",
			Args:     []any{user, info_type, value},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *WreviveExample) ExecSetUserInfo(
	user types.H160, info_type byte, value uint32, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSetUserInfo(user, info_type, value, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x1adb3149",
			Args:     []any{user, info_type, value},
		},
		__ink_params,
	)
}

func (c *WreviveExample) CallOfSetUserInfo(
	user types.H160, info_type byte, value uint32, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSetUserInfo(user, info_type, value, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x1adb3149",
			Args:     []any{user, info_type, value},
		},
	)
}

func (c *WreviveExample) QueryGetUserInfo(
	user types.H160, info_type byte, __ink_params chain.DryRunParams,
) (*uint32, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "get_user_info")
	}
	v, gas, err := chain.DryRunInk[uint32](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xd67b3f6f",
			Args:     []any{user, info_type},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) DryRunTransferBalance(
	from types.H160, to types.H160, amount uint64, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "transfer_balance")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xc27b120d",
			Args:     []any{from, to, amount},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *WreviveExample) ExecTransferBalance(
	from types.H160, to types.H160, amount uint64, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunTransferBalance(from, to, amount, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xc27b120d",
			Args:     []any{from, to, amount},
		},
		__ink_params,
	)
}

func (c *WreviveExample) CallOfTransferBalance(
	from types.H160, to types.H160, amount uint64, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunTransferBalance(from, to, amount, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xc27b120d",
			Args:     []any{from, to, amount},
		},
	)
}

func (c *WreviveExample) DryRunRecordsPush(
	value uint64, __ink_params chain.DryRunParams,
) (*util.Option[uint32], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "records_push")
	}
	v, gas, err := chain.DryRunInk[util.Option[uint32]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xcabd6b4b",
			Args:     []any{value},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) ExecRecordsPush(
	value uint64, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunRecordsPush(value, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xcabd6b4b",
			Args:     []any{value},
		},
		__ink_params,
	)
}

func (c *WreviveExample) CallOfRecordsPush(
	value uint64, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunRecordsPush(value, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xcabd6b4b",
			Args:     []any{value},
		},
	)
}

func (c *WreviveExample) QueryRecordsGet(
	id uint32, __ink_params chain.DryRunParams,
) (*uint64, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "records_get")
	}
	v, gas, err := chain.DryRunInk[uint64](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x17478972",
			Args:     []any{id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) QueryRecordsLen(
	__ink_params chain.DryRunParams,
) (*uint32, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "records_len")
	}
	v, gas, err := chain.DryRunInk[uint32](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xac77655a",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) QueryRecordsList(
	start uint32, size uint32, __ink_params chain.DryRunParams,
) (*[]Tuple_27, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "records_list")
	}
	v, gas, err := chain.DryRunInk[[]Tuple_27](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xaad263c8",
			Args:     []any{start, size},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) DryRunUserItemsPush(
	user types.H160, value uint32, __ink_params chain.DryRunParams,
) (*util.Option[uint32], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "user_items_push")
	}
	v, gas, err := chain.DryRunInk[util.Option[uint32]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x654b3f39",
			Args:     []any{user, value},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) ExecUserItemsPush(
	user types.H160, value uint32, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunUserItemsPush(user, value, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x654b3f39",
			Args:     []any{user, value},
		},
		__ink_params,
	)
}

func (c *WreviveExample) CallOfUserItemsPush(
	user types.H160, value uint32, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunUserItemsPush(user, value, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x654b3f39",
			Args:     []any{user, value},
		},
	)
}

func (c *WreviveExample) QueryUserItemsGet(
	user types.H160, k2 uint32, __ink_params chain.DryRunParams,
) (*uint32, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "user_items_get")
	}
	v, gas, err := chain.DryRunInk[uint32](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xe5d56223",
			Args:     []any{user, k2},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) QueryUserItemsLen(
	user types.H160, __ink_params chain.DryRunParams,
) (*uint32, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "user_items_len")
	}
	v, gas, err := chain.DryRunInk[uint32](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xfc510712",
			Args:     []any{user},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *WreviveExample) QueryUserItemsList(
	user types.H160, start uint32, size uint32, __ink_params chain.DryRunParams,
) (*[]Tuple_30, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "user_items_list")
	}
	v, gas, err := chain.DryRunInk[[]Tuple_30](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x33e26ec7",
			Args:     []any{user, start, size},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}
