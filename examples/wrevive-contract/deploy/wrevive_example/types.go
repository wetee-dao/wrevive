package wrevive_example

import (
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/wetee-dao/ink.go/util"
)

type Error struct { // Enum
	InsufficientBalance *bool // 0
	Unauthorized        *bool // 1
}

func (ty Error) Encode(encoder scale.Encoder) (err error) {
	if ty.InsufficientBalance != nil {
		err = encoder.PushByte(0)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.Unauthorized != nil {
		err = encoder.PushByte(1)
		if err != nil {
			return err
		}
		return nil
	}
	return fmt.Errorf("unrecognized enum")
}

func (ty *Error) Decode(decoder scale.Decoder) (err error) {
	variant, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}
	switch variant {
	case 0: // Base
		t := true
		ty.InsufficientBalance = &t
		return
	case 1: // Base
		t := true
		ty.Unauthorized = &t
		return
	default:
		return fmt.Errorf("unrecognized enum")
	}
}
func (ty *Error) Error() string {
	if ty.InsufficientBalance != nil {
		return "InsufficientBalance"
	}

	if ty.Unauthorized != nil {
		return "Unauthorized"
	}
	return "Unknown"
}

type Ip struct { // Composite
	Ipv4   util.Option[uint32]
	Ipv6   util.Option[types.U128]
	Domain util.Option[[]byte]
}
type Cluster struct { // Composite
	Name          util.Bytes
	Owner         types.H160
	Level         byte
	RegionId      uint32
	StartBlock    util.BlockNumber
	StopBlock     util.Option[util.BlockNumber]
	TerminalBlock util.Option[util.BlockNumber]
	P2pId         util.AccountId
	Ip            Ip
	Port          uint32
	Status        byte
}
type Tuple_26 struct { // Tuple
	F0 uint32
	F1 uint64
}
type Tuple_29 struct { // Tuple
	F0 uint32
	F1 uint32
}
