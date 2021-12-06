use num_bigint::{BigInt, BigUint};
use ton_abi::{Token, TokenValue};
use ton_block::{MsgAddrStd, MsgAddress, MsgAddressInt};
use ton_types::{BuilderData, Cell};

use super::{KnownParamType, Maybe, StandaloneToken};

pub trait PackAbiPlain {
    fn pack(self) -> Vec<Token>;
}

pub trait PackAbi: BuildTokenValue {
    fn pack(self) -> TokenValue;
}

pub trait BuildTokenValue {
    fn token_value(self) -> TokenValue;
}

impl BuildTokenValue for i8 {
    fn token_value(self) -> TokenValue {
        TokenValue::Int(ton_abi::Int {
            number: BigInt::from(self),
            size: 8,
        })
    }
}

impl BuildTokenValue for u8 {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: BigUint::from(self),
            size: 8,
        })
    }
}

impl BuildTokenValue for u16 {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: BigUint::from(self),
            size: 16,
        })
    }
}

impl BuildTokenValue for u32 {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: BigUint::from(self),
            size: 32,
        })
    }
}

impl BuildTokenValue for u64 {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: BigUint::from(self),
            size: 64,
        })
    }
}

impl BuildTokenValue for u128 {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: BigUint::from(self),
            size: 128,
        })
    }
}

impl BuildTokenValue for bool {
    fn token_value(self) -> TokenValue {
        TokenValue::Bool(self)
    }
}

impl BuildTokenValue for Cell {
    fn token_value(self) -> TokenValue {
        TokenValue::Cell(self)
    }
}

impl BuildTokenValue for MsgAddressInt {
    fn token_value(self) -> TokenValue {
        TokenValue::Address(match self {
            MsgAddressInt::AddrStd(addr) => MsgAddress::AddrStd(addr),
            MsgAddressInt::AddrVar(addr) => MsgAddress::AddrVar(addr),
        })
    }
}

impl BuildTokenValue for MsgAddrStd {
    fn token_value(self) -> TokenValue {
        TokenValue::Address(MsgAddress::AddrStd(self))
    }
}

impl BuildTokenValue for &str {
    fn token_value(self) -> TokenValue {
        TokenValue::Bytes(self.as_bytes().into())
    }
}

impl BuildTokenValue for Vec<u8> {
    fn token_value(self) -> TokenValue {
        TokenValue::Bytes(self)
    }
}

impl BuildTokenValue for BuilderData {
    fn token_value(self) -> TokenValue {
        TokenValue::Cell(self.into())
    }
}

impl<T> BuildTokenValue for Maybe<T>
where
    T: BuildTokenValue,
    Maybe<T>: KnownParamType,
{
    fn token_value(self) -> TokenValue {
        TokenValue::Optional(
            Self::param_type(),
            self.0.map(|item| Box::new(item.token_value())),
        )
    }
}

impl<T> BuildTokenValue for Vec<T>
where
    T: StandaloneToken + KnownParamType + BuildTokenValue,
{
    fn token_value(self) -> TokenValue {
        TokenValue::Array(
            T::param_type(),
            self.into_iter().map(BuildTokenValue::token_value).collect(),
        )
    }
}

impl BuildTokenValue for TokenValue {
    fn token_value(self) -> TokenValue {
        self
    }
}

impl<T> BuildTokenValue for &T
where
    T: Clone + BuildTokenValue,
{
    fn token_value(self) -> TokenValue {
        self.clone().token_value()
    }
}
