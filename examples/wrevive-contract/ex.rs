#![feature(prelude_import)]
//! Example contract using wrevive-api: Storage, Mapping, List, List2D with SCALE codec.
//! 示例合约：使用 wrevive-api 的 Storage、Mapping、List、List2D，采用 SCALE 编解码。
#![no_std]
#![no_main]
extern crate core;
#[prelude_import]
use core::prelude::rust_2024::*;
extern crate alloc;
mod datas {
    use parity_scale_codec::{Decode, Encode};
    use wrevive_api::{AccountId, Address, BlockNumber, Bytes, Vec};
    /// K8s worker 节点信息（Subnet::worker 返回值等）
    pub struct Cluster {
        pub name: Bytes,
        pub owner: Address,
        pub level: u8,
        pub region_id: u32,
        pub start_block: BlockNumber,
        pub stop_block: Option<BlockNumber>,
        pub terminal_block: Option<BlockNumber>,
        pub p2p_id: AccountId,
        pub ip: Ip,
        pub port: u32,
        pub status: u8,
    }
    #[automatically_derived]
    impl ::core::default::Default for Cluster {
        #[inline]
        fn default() -> Cluster {
            Cluster {
                name: ::core::default::Default::default(),
                owner: ::core::default::Default::default(),
                level: ::core::default::Default::default(),
                region_id: ::core::default::Default::default(),
                start_block: ::core::default::Default::default(),
                stop_block: ::core::default::Default::default(),
                terminal_block: ::core::default::Default::default(),
                p2p_id: ::core::default::Default::default(),
                ip: ::core::default::Default::default(),
                port: ::core::default::Default::default(),
                status: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Cluster {
        #[inline]
        fn clone(&self) -> Cluster {
            Cluster {
                name: ::core::clone::Clone::clone(&self.name),
                owner: ::core::clone::Clone::clone(&self.owner),
                level: ::core::clone::Clone::clone(&self.level),
                region_id: ::core::clone::Clone::clone(&self.region_id),
                start_block: ::core::clone::Clone::clone(&self.start_block),
                stop_block: ::core::clone::Clone::clone(&self.stop_block),
                terminal_block: ::core::clone::Clone::clone(&self.terminal_block),
                p2p_id: ::core::clone::Clone::clone(&self.p2p_id),
                ip: ::core::clone::Clone::clone(&self.ip),
                port: ::core::clone::Clone::clone(&self.port),
                status: ::core::clone::Clone::clone(&self.status),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Cluster {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Cluster {
        #[inline]
        fn eq(&self, other: &Cluster) -> bool {
            self.level == other.level && self.region_id == other.region_id
                && self.port == other.port && self.status == other.status
                && self.name == other.name && self.owner == other.owner
                && self.start_block == other.start_block
                && self.stop_block == other.stop_block
                && self.terminal_block == other.terminal_block
                && self.p2p_id == other.p2p_id && self.ip == other.ip
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Cluster {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<Bytes>;
            let _: ::core::cmp::AssertParamIsEq<Address>;
            let _: ::core::cmp::AssertParamIsEq<u8>;
            let _: ::core::cmp::AssertParamIsEq<u32>;
            let _: ::core::cmp::AssertParamIsEq<BlockNumber>;
            let _: ::core::cmp::AssertParamIsEq<Option<BlockNumber>>;
            let _: ::core::cmp::AssertParamIsEq<Option<BlockNumber>>;
            let _: ::core::cmp::AssertParamIsEq<AccountId>;
            let _: ::core::cmp::AssertParamIsEq<Ip>;
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Cluster {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "name",
                "owner",
                "level",
                "region_id",
                "start_block",
                "stop_block",
                "terminal_block",
                "p2p_id",
                "ip",
                "port",
                "status",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.name,
                &self.owner,
                &self.level,
                &self.region_id,
                &self.start_block,
                &self.stop_block,
                &self.terminal_block,
                &self.p2p_id,
                &self.ip,
                &self.port,
                &&self.status,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "Cluster",
                names,
                values,
            )
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Encode for Cluster {
            fn size_hint(&self) -> usize {
                0_usize
                    .saturating_add(::parity_scale_codec::Encode::size_hint(&self.name))
                    .saturating_add(::parity_scale_codec::Encode::size_hint(&self.owner))
                    .saturating_add(::parity_scale_codec::Encode::size_hint(&self.level))
                    .saturating_add(
                        ::parity_scale_codec::Encode::size_hint(&self.region_id),
                    )
                    .saturating_add(
                        ::parity_scale_codec::Encode::size_hint(&self.start_block),
                    )
                    .saturating_add(
                        ::parity_scale_codec::Encode::size_hint(&self.stop_block),
                    )
                    .saturating_add(
                        ::parity_scale_codec::Encode::size_hint(&self.terminal_block),
                    )
                    .saturating_add(
                        ::parity_scale_codec::Encode::size_hint(&self.p2p_id),
                    )
                    .saturating_add(::parity_scale_codec::Encode::size_hint(&self.ip))
                    .saturating_add(::parity_scale_codec::Encode::size_hint(&self.port))
                    .saturating_add(
                        ::parity_scale_codec::Encode::size_hint(&self.status),
                    )
            }
            fn encode_to<
                __CodecOutputEdqy: ::parity_scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                ::parity_scale_codec::Encode::encode_to(&self.name, __codec_dest_edqy);
                ::parity_scale_codec::Encode::encode_to(&self.owner, __codec_dest_edqy);
                ::parity_scale_codec::Encode::encode_to(&self.level, __codec_dest_edqy);
                ::parity_scale_codec::Encode::encode_to(
                    &self.region_id,
                    __codec_dest_edqy,
                );
                ::parity_scale_codec::Encode::encode_to(
                    &self.start_block,
                    __codec_dest_edqy,
                );
                ::parity_scale_codec::Encode::encode_to(
                    &self.stop_block,
                    __codec_dest_edqy,
                );
                ::parity_scale_codec::Encode::encode_to(
                    &self.terminal_block,
                    __codec_dest_edqy,
                );
                ::parity_scale_codec::Encode::encode_to(&self.p2p_id, __codec_dest_edqy);
                ::parity_scale_codec::Encode::encode_to(&self.ip, __codec_dest_edqy);
                ::parity_scale_codec::Encode::encode_to(&self.port, __codec_dest_edqy);
                ::parity_scale_codec::Encode::encode_to(&self.status, __codec_dest_edqy);
            }
        }
        #[automatically_derived]
        impl ::parity_scale_codec::EncodeLike for Cluster {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Decode for Cluster {
            fn decode<__CodecInputEdqy: ::parity_scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::parity_scale_codec::Error> {
                ::core::result::Result::Ok(Cluster {
                    name: {
                        let __codec_res_edqy = <Bytes as ::parity_scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Cluster::name`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    owner: {
                        let __codec_res_edqy = <Address as ::parity_scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Cluster::owner`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    level: {
                        let __codec_res_edqy = <u8 as ::parity_scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Cluster::level`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    region_id: {
                        let __codec_res_edqy = <u32 as ::parity_scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Cluster::region_id`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    start_block: {
                        let __codec_res_edqy = <BlockNumber as ::parity_scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Cluster::start_block`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    stop_block: {
                        let __codec_res_edqy = <Option<
                            BlockNumber,
                        > as ::parity_scale_codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Cluster::stop_block`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    terminal_block: {
                        let __codec_res_edqy = <Option<
                            BlockNumber,
                        > as ::parity_scale_codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Cluster::terminal_block`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    p2p_id: {
                        let __codec_res_edqy = <AccountId as ::parity_scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Cluster::p2p_id`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    ip: {
                        let __codec_res_edqy = <Ip as ::parity_scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Cluster::ip`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    port: {
                        let __codec_res_edqy = <u32 as ::parity_scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Cluster::port`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    status: {
                        let __codec_res_edqy = <u8 as ::parity_scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Cluster::status`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                })
            }
        }
    };
    /// Ip
    pub struct Ip {
        pub ipv4: Option<u32>,
        pub ipv6: Option<u128>,
        pub domain: Option<Vec<u8>>,
    }
    #[automatically_derived]
    impl ::core::default::Default for Ip {
        #[inline]
        fn default() -> Ip {
            Ip {
                ipv4: ::core::default::Default::default(),
                ipv6: ::core::default::Default::default(),
                domain: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Ip {
        #[inline]
        fn clone(&self) -> Ip {
            Ip {
                ipv4: ::core::clone::Clone::clone(&self.ipv4),
                ipv6: ::core::clone::Clone::clone(&self.ipv6),
                domain: ::core::clone::Clone::clone(&self.domain),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Ip {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Ip {
        #[inline]
        fn eq(&self, other: &Ip) -> bool {
            self.ipv4 == other.ipv4 && self.ipv6 == other.ipv6
                && self.domain == other.domain
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Ip {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<Option<u32>>;
            let _: ::core::cmp::AssertParamIsEq<Option<u128>>;
            let _: ::core::cmp::AssertParamIsEq<Option<Vec<u8>>>;
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Ip {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "Ip",
                "ipv4",
                &self.ipv4,
                "ipv6",
                &self.ipv6,
                "domain",
                &&self.domain,
            )
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Encode for Ip {
            fn size_hint(&self) -> usize {
                0_usize
                    .saturating_add(::parity_scale_codec::Encode::size_hint(&self.ipv4))
                    .saturating_add(::parity_scale_codec::Encode::size_hint(&self.ipv6))
                    .saturating_add(
                        ::parity_scale_codec::Encode::size_hint(&self.domain),
                    )
            }
            fn encode_to<
                __CodecOutputEdqy: ::parity_scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                ::parity_scale_codec::Encode::encode_to(&self.ipv4, __codec_dest_edqy);
                ::parity_scale_codec::Encode::encode_to(&self.ipv6, __codec_dest_edqy);
                ::parity_scale_codec::Encode::encode_to(&self.domain, __codec_dest_edqy);
            }
        }
        #[automatically_derived]
        impl ::parity_scale_codec::EncodeLike for Ip {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Decode for Ip {
            fn decode<__CodecInputEdqy: ::parity_scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::parity_scale_codec::Error> {
                ::core::result::Result::Ok(Ip {
                    ipv4: {
                        let __codec_res_edqy = <Option<
                            u32,
                        > as ::parity_scale_codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Ip::ipv4`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    ipv6: {
                        let __codec_res_edqy = <Option<
                            u128,
                        > as ::parity_scale_codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Ip::ipv6`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    domain: {
                        let __codec_res_edqy = <Option<
                            Vec<u8>,
                        > as ::parity_scale_codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Ip::domain`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                })
            }
        }
    };
}
static ALLOC: pvm_bump_allocator::BumpAllocator<1024> = pvm_bump_allocator::BumpAllocator::new();
const _: () = {
    #[rustc_std_internal_symbol]
    #[rustc_allocator]
    unsafe fn __rust_alloc(size: usize, align: usize) -> *mut u8 {
        ::core::alloc::GlobalAlloc::alloc(
            &ALLOC,
            ::core::alloc::Layout::from_size_align_unchecked(size, align),
        )
    }
    #[rustc_std_internal_symbol]
    #[rustc_deallocator]
    unsafe fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize) -> () {
        ::core::alloc::GlobalAlloc::dealloc(
            &ALLOC,
            ptr,
            ::core::alloc::Layout::from_size_align_unchecked(size, align),
        )
    }
    #[rustc_std_internal_symbol]
    #[rustc_reallocator]
    unsafe fn __rust_realloc(
        ptr: *mut u8,
        size: usize,
        align: usize,
        new_size: usize,
    ) -> *mut u8 {
        ::core::alloc::GlobalAlloc::realloc(
            &ALLOC,
            ptr,
            ::core::alloc::Layout::from_size_align_unchecked(size, align),
            new_size,
        )
    }
    #[rustc_std_internal_symbol]
    #[rustc_allocator_zeroed]
    unsafe fn __rust_alloc_zeroed(size: usize, align: usize) -> *mut u8 {
        ::core::alloc::GlobalAlloc::alloc_zeroed(
            &ALLOC,
            ::core::alloc::Layout::from_size_align_unchecked(size, align),
        )
    }
};
use wrevive_api::{env, Address, Encode, Env, List, List2D, Mapping, Storage, Vec};
use wrevive_macro::{list, list_2d, mapping, revive_contract, storage};
pub use datas::*;
use wrevive_api::Decode;
use pvm_contract_types::{SolDecode, SolEncode};
pub mod contract {
    use super::*;
    use wrevive_api::Env;
    /// Event topics (empty for simple events). 事件主题（简单事件可为空）。
    const EMPTY_TOPICS: &[[u8; 32]] = &[];
    /// Single value storage; prefix = Blake2s256(b"value")[0..4].
    /// 单值存储；prefix 由 storage! 宏用 Blake2s256 取前 4 字节。
    const VALUE: Storage<u32> = wrevive_api::Storage::new(&[217u8, 202u8, 11u8, 229u8]);
    /// Contract owner address (20 bytes). 合约所有者地址（20 字节）。
    const OWNER: Storage<Address> = wrevive_api::Storage::new(
        &[156u8, 224u8, 129u8, 130u8],
    );
    /// Cluster info; prefix = Blake2s256(b"cluster")[0..4].
    const CLUSTER: Storage<Cluster> = wrevive_api::Storage::new(
        &[221u8, 76u8, 5u8, 84u8],
    );
    /// Balance per account: key = Address, value = u64.
    /// 用户余额：key = 用户地址，value = 余额。
    const BALANCE_MAPPING: Mapping<Address, u64> = wrevive_api::Mapping::new(
        &[1u8, 127u8, 143u8, 161u8],
    );
    /// User info by (address, info_type): value = u32 (e.g. score, level).
    /// 用户信息：key = (地址, 类型)，value = u32（如积分、等级）。
    const USER_INFO_MAPPING: Mapping<(Address, u8), u32> = wrevive_api::Mapping::new(
        &[79u8, 125u8, 141u8, 117u8],
    );
    /// Global list: auto-increment id (u32), value u64. 全局列表：自增 id(u32)，值 u64。
    const RECORDS: List<u32, u64> = wrevive_api::List::new(
        &[173u8, 17u8, 7u8, 143u8],
        &[23u8, 226u8, 77u8, 91u8],
    );
    /// Per-user list: each Address has a list of u32. 按用户维度的列表：每用户一条 u32 列表。
    const USER_ITEMS: List2D<Address, u32, u32> = wrevive_api::List2D::new(
        &[252u8, 59u8, 65u8, 119u8],
        &[26u8, 95u8, 146u8, 66u8],
        &[21u8, 123u8, 96u8, 234u8],
        &[166u8, 214u8, 124u8, 145u8],
    );
    /// Contract error type. 合约错误类型。
    pub enum Error {
        InsufficientBalance,
        Unauthorized,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Error {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    Error::InsufficientBalance => "InsufficientBalance",
                    Error::Unauthorized => "Unauthorized",
                },
            )
        }
    }
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for Error {}
    #[automatically_derived]
    impl ::core::clone::Clone for Error {
        #[inline]
        fn clone(&self) -> Error {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Error {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Error {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Error {
        #[inline]
        fn eq(&self, other: &Error) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Error {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) {}
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Encode for Error {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        Error::InsufficientBalance => 0_usize,
                        Error::Unauthorized => 0_usize,
                        _ => 0_usize,
                    }
            }
            fn encode_to<
                __CodecOutputEdqy: ::parity_scale_codec::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                #[automatically_derived]
                const _: () = {
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    const indices: [(usize, &'static str); 2usize] = [
                        ((0usize) as ::core::primitive::usize, "InsufficientBalance"),
                        ((1usize) as ::core::primitive::usize, "Unauthorized"),
                    ];
                    const fn search_for_invalid_index(
                        array: &[(usize, &'static str); 2usize],
                    ) -> (bool, usize) {
                        let mut i = 0;
                        while i < 2usize {
                            if array[i].0 > 255 {
                                return (true, i);
                            }
                            i += 1;
                        }
                        (false, 0)
                    }
                    const INVALID_INDEX: (bool, usize) = search_for_invalid_index(
                        &indices,
                    );
                    if INVALID_INDEX.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper("Found variant `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].1,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` with invalid index: `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].0,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Max supported index is 255.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            ::core::panicking::panic_display(&msg);
                        };
                    }
                    const fn duplicate_info(
                        array: &[(usize, &'static str); 2usize],
                    ) -> (bool, usize, usize) {
                        let len = 2usize;
                        let mut i = 0usize;
                        while i < len {
                            let mut j = i + 1;
                            while j < len {
                                if array[i].0 == array[j].0 {
                                    return (true, i, j);
                                }
                                j += 1;
                            }
                            i += 1;
                        }
                        (false, 0, 0)
                    }
                    const DUP_INFO: (bool, usize, usize) = duplicate_info(&indices);
                    if DUP_INFO.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "Found variants that have duplicate indexes. Both `",
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` and `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.2].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` have the index `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].0)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Use different indexes for each variant.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            ::core::panicking::panic_display(&msg);
                        };
                    }
                };
                match *self {
                    Error::InsufficientBalance => {
                        #[allow(clippy::unnecessary_cast)]
                        #[allow(clippy::cast_possible_truncation)]
                        __codec_dest_edqy.push_byte((0usize) as ::core::primitive::u8);
                    }
                    Error::Unauthorized => {
                        #[allow(clippy::unnecessary_cast)]
                        #[allow(clippy::cast_possible_truncation)]
                        __codec_dest_edqy.push_byte((1usize) as ::core::primitive::u8);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl ::parity_scale_codec::EncodeLike for Error {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::parity_scale_codec::Decode for Error {
            fn decode<__CodecInputEdqy: ::parity_scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::parity_scale_codec::Error> {
                #[automatically_derived]
                const _: () = {
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    const indices: [(usize, &'static str); 2usize] = [
                        ((0usize) as ::core::primitive::usize, "InsufficientBalance"),
                        ((1usize) as ::core::primitive::usize, "Unauthorized"),
                    ];
                    const fn search_for_invalid_index(
                        array: &[(usize, &'static str); 2usize],
                    ) -> (bool, usize) {
                        let mut i = 0;
                        while i < 2usize {
                            if array[i].0 > 255 {
                                return (true, i);
                            }
                            i += 1;
                        }
                        (false, 0)
                    }
                    const INVALID_INDEX: (bool, usize) = search_for_invalid_index(
                        &indices,
                    );
                    if INVALID_INDEX.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper("Found variant `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].1,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` with invalid index: `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    indices[INVALID_INDEX.1].0,
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Max supported index is 255.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            ::core::panicking::panic_display(&msg);
                        };
                    }
                    const fn duplicate_info(
                        array: &[(usize, &'static str); 2usize],
                    ) -> (bool, usize, usize) {
                        let len = 2usize;
                        let mut i = 0usize;
                        while i < len {
                            let mut j = i + 1;
                            while j < len {
                                if array[i].0 == array[j].0 {
                                    return (true, i, j);
                                }
                                j += 1;
                            }
                            i += 1;
                        }
                        (false, 0, 0)
                    }
                    const DUP_INFO: (bool, usize, usize) = duplicate_info(&indices);
                    if DUP_INFO.0 {
                        let msg = ::const_format::pmr::__AssertStr {
                            x: {
                                use ::const_format::__cf_osRcTFl4A;
                                ({
                                    #[doc(hidden)]
                                    #[allow(unused_mut, non_snake_case)]
                                    const CONCATP_NHPMWYD3NJA: &[__cf_osRcTFl4A::pmr::PArgument] = {
                                        let fmt = __cf_osRcTFl4A::pmr::FormattingFlags::NEW;
                                        &[
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "Found variants that have duplicate indexes. Both `",
                                                )
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` and `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.2].1)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper("` have the index `")
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(indices[DUP_INFO.1].0)
                                                .to_pargument_display(fmt),
                                            __cf_osRcTFl4A::pmr::PConvWrapper(
                                                    "`. Use different indexes for each variant.",
                                                )
                                                .to_pargument_display(fmt),
                                        ]
                                    };
                                    {
                                        #[doc(hidden)]
                                        const ARR_LEN: usize = ::const_format::pmr::PArgument::calc_len(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        const CONCAT_ARR: &::const_format::pmr::LenAndArray<
                                            [u8; ARR_LEN],
                                        > = &::const_format::pmr::__priv_concatenate(
                                            CONCATP_NHPMWYD3NJA,
                                        );
                                        #[doc(hidden)]
                                        #[allow(clippy::transmute_ptr_to_ptr)]
                                        const CONCAT_STR: &str = unsafe {
                                            let slice = ::const_format::pmr::transmute::<
                                                &[u8; ARR_LEN],
                                                &[u8; CONCAT_ARR.len],
                                            >(&CONCAT_ARR.array);
                                            {
                                                let bytes: &'static [::const_format::pmr::u8] = slice;
                                                let string: &'static ::const_format::pmr::str = {
                                                    ::const_format::__hidden_utils::PtrToRef {
                                                        ptr: bytes as *const [::const_format::pmr::u8] as *const str,
                                                    }
                                                        .reff
                                                };
                                                string
                                            }
                                        };
                                        CONCAT_STR
                                    }
                                })
                            },
                        }
                            .x;
                        {
                            ::core::panicking::panic_display(&msg);
                        };
                    }
                };
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| {
                        e.chain("Could not decode `Error`, failed to read variant byte")
                    })?
                {
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    __codec_x_edqy if __codec_x_edqy
                        == (0usize) as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(Error::InsufficientBalance)
                        })();
                    }
                    #[allow(clippy::unnecessary_cast)]
                    #[allow(clippy::cast_possible_truncation)]
                    __codec_x_edqy if __codec_x_edqy
                        == (1usize) as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(Error::Unauthorized)
                        })();
                    }
                    _ => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Err(
                                <_ as ::core::convert::Into<
                                    _,
                                >>::into("Could not decode `Error`, variant doesn't exist"),
                            )
                        })();
                    }
                }
            }
        }
    };
    /// Constructor: set caller as owner and init VALUE to the given initial_value.
    /// 构造函数：设置调用者为 owner，VALUE 初始为 initial_value。
    ///
    /// # Parameters
    /// @param initial_value The initial value to set in the contract
    /// @param initial_value 合约初始化时设置的初始值
    ///
    /// # Returns
    /// @return Returns Ok(()) if initialization succeeds, Err(Error) if failed
    /// @return 初始化成功返回 Ok(())，失败返回 Err(Error)
    ///
    /// # Events
    /// Emits an event with the initial value when deployment succeeds
    /// 部署成功时发送包含初始值的事件
    pub fn deploy(initial_value: u32) -> Result<(), Error> {
        VALUE.set(&initial_value);
        OWNER.set(&env().caller());
        env().deposit_event(EMPTY_TOPICS, &initial_value.to_le_bytes().as_slice());
        Ok(())
    }
    /// Set the contract's stored value.
    /// 设置合约中存储的值。
    ///
    /// # Parameters
    /// @param value The new value to store in the contract
    /// @param value 要在合约中存储的新值
    ///
    /// # Returns
    /// @return Returns Ok(()) if value is set successfully
    /// @return 成功设置值返回 Ok(())
    ///
    /// # Events
    /// Emits an event with the new value
    /// 发送包含新值的事件
    pub fn set_value(value: u32) -> Result<(), Error> {
        VALUE.set(&value);
        env().deposit_event(EMPTY_TOPICS, &value.to_le_bytes().as_slice());
        Ok(())
    }
    /// Get the current stored value.
    /// 获取当前存储的值。
    ///
    /// # Returns
    /// @return Returns the current stored value, defaults to 0 if not set
    /// @return 返回当前存储的值，未设置时默认为 0
    pub fn get_value() -> u32 {
        VALUE.get().unwrap_or(0)
    }
    /// Get the current stored value as Option.
    /// 以 Option 类型获取当前存储的值。
    ///
    /// # Returns
    /// @return Returns Some(value) if value is set, None otherwise
    /// @return 设置了值返回 Some(value)，否则返回 None
    pub fn get_value_option() -> Option<u32> {
        VALUE.get()
    }
    /// Set value using Solidity encoding; only current owner may call (else revert).
    /// 使用 Solidity 编码设置值；仅当前所有者可调用，否则 revert。
    ///
    /// # Parameters
    /// @param value The new value to store (Solidity encoded)
    /// @param value 要存储的新值（Solidity 编码）
    ///
    /// # Returns
    /// @return Returns Ok(()) if value is set successfully
    /// @return 成功设置值返回 Ok(())
    pub fn set_value_sol(value: u32) -> Result<(), Error> {
        VALUE.set(&value);
        env().deposit_event(EMPTY_TOPICS, &value.to_le_bytes().as_slice());
        Ok(())
    }
    /// Get value using Solidity encoding; only current owner may call (else revert).
    /// 使用 Solidity 编码获取值；仅当前所有者可调用，否则 revert。
    ///
    /// # Returns
    /// @return Returns the current stored value (Solidity encoded)
    /// @return 返回当前存储的值（Solidity 编码）
    pub fn get_value_sol() -> u32 {
        VALUE.get().unwrap_or(0)
    }
    /// Get the cluster information stored in the contract.
    /// 获取合约中存储的集群信息。
    ///
    /// # Returns
    /// @return Returns the cluster info, defaults to Cluster::default() if not set
    /// @return 返回集群信息，未设置时默认为 Cluster::default()
    pub fn get_cluster() -> Cluster {
        CLUSTER.get().unwrap_or(Cluster::default())
    }
    /// Set cluster information in the contract.
    /// 在合约中设置集群信息。
    ///
    /// # Parameters
    /// @param cluster The cluster information to store
    /// @param cluster 要存储的集群信息
    ///
    /// # Returns
    /// @return Returns Ok(()) if cluster is set successfully
    /// @return 成功设置集群信息返回 Ok(())
    pub fn set_cluster(cluster: Cluster) -> Result<(), Error> {
        CLUSTER.set(&cluster);
        Ok(())
    }
    /// Set new contract owner. Only current owner may call (else revert).
    /// 设置新的合约所有者。仅当前所有者可调用，否则 revert。
    ///
    /// # Parameters
    /// @param new_owner The address of the new contract owner
    /// @param new_owner 新的合约所有者地址
    ///
    /// # Returns
    /// @return Returns Ok(()) if ownership transfer succeeds, Err(Error::Unauthorized) if caller is not current owner
    /// @return 所有权转移成功返回 Ok(())，调用者不是当前所有者返回 Err(Error::Unauthorized)
    ///
    /// # Security
    /// Only the current contract owner can transfer ownership to prevent unauthorized changes
    /// 只有当前合约所有者才能转移所有权，防止未授权的更改
    pub fn set_owner(new_owner: Address) -> Result<(), Error> {
        let caller = env().caller();
        let current_owner = get_owner();
        if caller != current_owner {
            return Err(Error::Unauthorized);
        }
        OWNER.set(&new_owner);
        Ok(())
    }
    /// Get the current contract owner address.
    /// 获取当前合约所有者地址。
    ///
    /// # Returns
    /// @return Returns the current owner address, zero address if not set
    /// @return 返回当前所有者地址，未设置时返回零地址
    pub fn get_owner() -> Address {
        OWNER.get().unwrap_or(Address::zero())
    }
    /// Set balance for a specific user account using Mapping storage.
    /// 使用 Mapping 存储为特定用户账户设置余额。
    ///
    /// # Parameters
    /// @param user The user address to set balance for
    /// @param user 要设置余额的用户地址
    /// @param balance The new balance value for the user
    /// @param balance 用户的新余额值
    ///
    /// # Returns
    /// @return Returns Ok(()) if balance is set successfully
    /// @return 成功设置余额返回 Ok(())
    pub fn set_balance(user: Address, balance: u64) -> Result<(), Error> {
        BALANCE_MAPPING.set(&user, &balance);
        Ok(())
    }
    /// Get balance for a specific user account using Mapping storage.
    /// 使用 Mapping 存储获取特定用户账户的余额。
    ///
    /// # Parameters
    /// @param user The user address to get balance for
    /// @param user 要获取余额的用户地址
    ///
    /// # Returns
    /// @return Returns the user's current balance, defaults to 0 if not set
    /// @return 返回用户当前余额，未设置时默认为 0
    pub fn get_balance(user: Address) -> u64 {
        BALANCE_MAPPING.get(&user).unwrap_or(0)
    }
    /// Set user information with compound key (user address + info type).
    /// 使用复合键（用户地址 + 信息类型）设置用户信息。
    ///
    /// # Parameters
    /// @param user The user address
    /// @param user 用户地址
    /// @param info_type The type of user information (e.g., score=1, level=2)
    /// @param info_type 用户信息类型（如：积分=1，等级=2）
    /// @param value The information value to store
    /// @param value 要存储的信息值
    ///
    /// # Returns
    /// @return Returns Ok(()) if user info is set successfully
    /// @return 成功设置用户信息返回 Ok(())
    pub fn set_user_info(user: Address, info_type: u8, value: u32) -> Result<(), Error> {
        USER_INFO_MAPPING.set(&(user, info_type), &value);
        Ok(())
    }
    /// Get user information with compound key (user address + info type).
    /// 使用复合键（用户地址 + 信息类型）获取用户信息。
    ///
    /// # Parameters
    /// @param user The user address
    /// @param user 用户地址
    /// @param info_type The type of user information to retrieve
    /// @param info_type 要获取的用户信息类型
    ///
    /// # Returns
    /// @return Returns the user information value, defaults to 0 if not set
    /// @return 返回用户信息值，未设置时默认为 0
    pub fn get_user_info(user: Address, info_type: u8) -> u32 {
        USER_INFO_MAPPING.get(&(user, info_type)).unwrap_or(0)
    }
    /// Transfer balance from one account to another. Only the sender (`from`) may call (else revert).
    /// Reverts if `from` has insufficient balance. Self-transfer (from == to) is a no-op.
    /// 转账：仅 from 可发起；余额不足时 revert；from == to 时不操作。
    ///
    /// # Parameters
    /// @param from The source address to transfer from (must be caller)
    /// @param from 转出地址（必须是调用者）
    /// @param to The destination address to transfer to
    /// @param to 转入地址
    /// @param amount The amount to transfer
    /// @param amount 转账金额
    ///
    /// # Returns
    /// @return Returns Ok(()) if transfer succeeds
    /// @return Returns Err(Error::Unauthorized) if caller is not the from address
    /// @return Returns Err(Error::InsufficientBalance) if from address has insufficient balance
    /// @return 转账成功返回 Ok(())
    /// @return 调用者不是 from 地址返回 Err(Error::Unauthorized)
    /// @return from 地址余额不足返回 Err(Error::InsufficientBalance)
    ///
    /// # Security
    /// Only the 'from' address can initiate the transfer to prevent unauthorized transfers
    /// 只有 'from' 地址可以发起转账，防止未授权的转账
    ///
    /// # Edge Cases
    /// - Self-transfer (from == to) is treated as a no-op and returns Ok(())
    /// - Zero amount transfers are treated as no-ops
    /// - 自转账（from == to）被视为无操作并返回 Ok(())
    /// - 零金额转账被视为无操作
    pub fn transfer_balance(
        from: Address,
        to: Address,
        amount: u64,
    ) -> Result<(), Error> {
        let caller = env().caller();
        if caller != from {
            return Err(Error::Unauthorized);
        }
        if from == to || amount == 0 {
            return Ok(());
        }
        let from_balance = BALANCE_MAPPING.get(&from).unwrap_or(0);
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }
        let to_balance = BALANCE_MAPPING.get(&to).unwrap_or(0);
        BALANCE_MAPPING.set(&from, &(from_balance - amount));
        BALANCE_MAPPING.set(&to, &(to_balance + amount));
        Ok(())
    }
    /// 向全局 records 列表追加一条 u64，返回分配到的 id
    pub fn records_push(value: u64) -> Option<u32> {
        RECORDS.insert(&value)
    }
    /// 按 id 取 records 中的值
    pub fn records_get(id: u32) -> u64 {
        RECORDS.get(&id).unwrap_or(0)
    }
    /// 全局 records 长度
    pub fn records_len() -> u32 {
        RECORDS.len()
    }
    /// 分页：从 start 起取最多 size 条 (id, value)。返回长度 0 表示无数据或参数不合法
    pub fn records_list(start: u32, size: u32) -> Vec<(u32, u64)> {
        RECORDS.list(start, size)
    }
    /// 在指定用户下追加一条 u32，返回该用户下的 k2
    pub fn user_items_push(user: Address, value: u32) -> Option<u32> {
        USER_ITEMS.insert(&user, &value)
    }
    /// 取用户 user 下第 k2 条
    pub fn user_items_get(user: Address, k2: u32) -> u32 {
        USER_ITEMS.get(&user, k2).unwrap_or(0)
    }
    /// 用户 user 下的条目数量
    pub fn user_items_len(user: Address) -> u32 {
        USER_ITEMS.len(&user)
    }
    /// 分页：用户 user 下从 start 起取最多 size 条 (k2, value)
    pub fn user_items_list(user: Address, start: u32, size: u32) -> Vec<(u32, u32)> {
        USER_ITEMS.list(&user, start, size)
    }
    /// Contract-to-contract call interface module
    ///
    /// This module is automatically generated by #[revive_contract] macro to provide:
    /// 1. SELECTOR_* constants: 4-byte selectors for each message function
    /// 2. encode_* functions: encode function arguments to SCALE format
    /// 3. call_raw function: low-level contract call interface
    /// 4. call_* functions: convenient high-level call interfaces (encode+call+decode)
    /// 5. constructor's encode_* / instantiate_* functions
    ///
    /// # Usage Examples
    /// ```rust
    /// use crate::contract::api;
    ///
    /// // Method 1: Use high-level interface (recommended)
    /// let result = api::set_value(&callee_address, &42)?;
    ///
    /// // Method 2: Manual encoding + call
    /// let input = api::encode_set_value(&42);
    /// let raw_result = api::call_raw(&callee_address, &input)?;
    /// let decoded_result: Result<(), Error> = Decode::decode(&mut &raw_result[..])?;
    /// ```
    ///
    /// # Notes
    /// - Only functions using Codec encoding will generate encode_* and call_* functions
    /// - Sol encoded functions require manual encoding/decoding handling
    /// - Failed calls return ReturnErrorCode, which needs to be handled
    pub mod api {
        use super::*;
        use wrevive_api::*;
        /// Message function selector constants
        ///
        /// Each constant corresponds to a 4-byte selector of a message function,
        /// used to construct contract call input data.
        pub const SELECTOR_SET_VALUE: [u8; 4] = [175u8, 215u8, 144u8, 86u8];
        pub const SELECTOR_GET_VALUE: [u8; 4] = [194u8, 104u8, 19u8, 211u8];
        pub const SELECTOR_GET_VALUE_OPTION: [u8; 4] = [133u8, 229u8, 227u8, 189u8];
        pub const SELECTOR_SET_VALUE_SOL: [u8; 4] = [250u8, 198u8, 127u8, 177u8];
        pub const SELECTOR_GET_VALUE_SOL: [u8; 4] = [110u8, 45u8, 253u8, 179u8];
        pub const SELECTOR_GET_CLUSTER: [u8; 4] = [133u8, 16u8, 101u8, 115u8];
        pub const SELECTOR_SET_CLUSTER: [u8; 4] = [192u8, 156u8, 36u8, 6u8];
        pub const SELECTOR_SET_OWNER: [u8; 4] = [85u8, 63u8, 115u8, 79u8];
        pub const SELECTOR_GET_OWNER: [u8; 4] = [171u8, 238u8, 11u8, 250u8];
        pub const SELECTOR_SET_BALANCE: [u8; 4] = [139u8, 160u8, 143u8, 72u8];
        pub const SELECTOR_GET_BALANCE: [u8; 4] = [59u8, 158u8, 17u8, 212u8];
        pub const SELECTOR_SET_USER_INFO: [u8; 4] = [26u8, 219u8, 49u8, 73u8];
        pub const SELECTOR_GET_USER_INFO: [u8; 4] = [214u8, 123u8, 63u8, 111u8];
        pub const SELECTOR_TRANSFER_BALANCE: [u8; 4] = [194u8, 123u8, 18u8, 13u8];
        pub const SELECTOR_RECORDS_PUSH: [u8; 4] = [202u8, 189u8, 107u8, 75u8];
        pub const SELECTOR_RECORDS_GET: [u8; 4] = [23u8, 71u8, 137u8, 114u8];
        pub const SELECTOR_RECORDS_LEN: [u8; 4] = [172u8, 119u8, 101u8, 90u8];
        pub const SELECTOR_RECORDS_LIST: [u8; 4] = [170u8, 210u8, 99u8, 200u8];
        pub const SELECTOR_USER_ITEMS_PUSH: [u8; 4] = [101u8, 75u8, 63u8, 57u8];
        pub const SELECTOR_USER_ITEMS_GET: [u8; 4] = [229u8, 213u8, 98u8, 35u8];
        pub const SELECTOR_USER_ITEMS_LEN: [u8; 4] = [252u8, 81u8, 7u8, 18u8];
        pub const SELECTOR_USER_ITEMS_LIST: [u8; 4] = [51u8, 226u8, 110u8, 199u8];
        /// Constructor selector (Keccak-256 of function name; same 4 bytes `deploy()` skips before decoding).
        pub const SELECTOR_DEPLOY: [u8; 4] = [244u8, 230u8, 84u8, 160u8];
        /// Low-level contract call function
        ///
        /// Calls the specified contract with encoded input (4-byte selector + encoded parameters),
        /// returns the raw bytes from the contract.
        ///
        /// # Parameters
        /// * `callee` - Target contract address
        /// * `input` - Encoded call data (selector + parameter encoding)
        ///
        /// # Return Value
        /// Returns the raw return bytes from the contract, needs manual decoding
        ///
        /// # Error Handling
        /// Returns ReturnErrorCode on call failure
        #[inline(always)]
        pub fn call_raw(
            callee: &Address,
            input: &[u8],
        ) -> Result<Vec<u8>, ReturnErrorCode> {
            let r = wrevive_api::env()
                .call(
                    pallet_revive_uapi::CallFlags::empty(),
                    callee,
                    u64::MAX,
                    u64::MAX,
                    &wrevive_api::U256::ZERO,
                    &wrevive_api::U256::ZERO,
                    input,
                    None,
                );
            r.map_err(|e| e)?;
            let size = wrevive_api::env().return_data_size() as usize;
            let mut buf = ::alloc::vec::from_elem(0u8, size);
            let mut slice = buf.as_mut_slice();
            wrevive_api::env().return_data_copy(&mut slice, 0);
            Ok(buf)
        }
        /// Parameter encoding functions
        ///
        /// For each message function using Codec encoding, generates corresponding encode_* functions,
        /// used to encode function arguments into SCALE format byte sequences.
        #[inline(always)]
        pub fn encode_set_value(value: &u32) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[175u8, 215u8, 144u8, 86u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(value));
            out
        }
        #[inline(always)]
        pub fn encode_get_value() -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4);
            out.extend_from_slice(&[194u8, 104u8, 19u8, 211u8]);
            out
        }
        #[inline(always)]
        pub fn encode_get_value_option() -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4);
            out.extend_from_slice(&[133u8, 229u8, 227u8, 189u8]);
            out
        }
        #[inline(always)]
        pub fn encode_get_cluster() -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4);
            out.extend_from_slice(&[133u8, 16u8, 101u8, 115u8]);
            out
        }
        #[inline(always)]
        pub fn encode_set_cluster(cluster: &Cluster) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[192u8, 156u8, 36u8, 6u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(cluster));
            out
        }
        #[inline(always)]
        pub fn encode_set_owner(new_owner: &Address) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[85u8, 63u8, 115u8, 79u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(new_owner));
            out
        }
        #[inline(always)]
        pub fn encode_get_owner() -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4);
            out.extend_from_slice(&[171u8, 238u8, 11u8, 250u8]);
            out
        }
        #[inline(always)]
        pub fn encode_set_balance(user: &Address, balance: &u64) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[139u8, 160u8, 143u8, 72u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(user, balance)));
            out
        }
        #[inline(always)]
        pub fn encode_get_balance(user: &Address) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[59u8, 158u8, 17u8, 212u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(user));
            out
        }
        #[inline(always)]
        pub fn encode_set_user_info(
            user: &Address,
            info_type: &u8,
            value: &u32,
        ) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[26u8, 219u8, 49u8, 73u8]);
            out.extend_from_slice(
                &wrevive_api::Encode::encode(&(user, info_type, value)),
            );
            out
        }
        #[inline(always)]
        pub fn encode_get_user_info(
            user: &Address,
            info_type: &u8,
        ) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[214u8, 123u8, 63u8, 111u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(user, info_type)));
            out
        }
        #[inline(always)]
        pub fn encode_transfer_balance(
            from: &Address,
            to: &Address,
            amount: &u64,
        ) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[194u8, 123u8, 18u8, 13u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(from, to, amount)));
            out
        }
        #[inline(always)]
        pub fn encode_records_push(value: &u64) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[202u8, 189u8, 107u8, 75u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(value));
            out
        }
        #[inline(always)]
        pub fn encode_records_get(id: &u32) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[23u8, 71u8, 137u8, 114u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(id));
            out
        }
        #[inline(always)]
        pub fn encode_records_len() -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4);
            out.extend_from_slice(&[172u8, 119u8, 101u8, 90u8]);
            out
        }
        #[inline(always)]
        pub fn encode_records_list(start: &u32, size: &u32) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[170u8, 210u8, 99u8, 200u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(start, size)));
            out
        }
        #[inline(always)]
        pub fn encode_user_items_push(
            user: &Address,
            value: &u32,
        ) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[101u8, 75u8, 63u8, 57u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(user, value)));
            out
        }
        #[inline(always)]
        pub fn encode_user_items_get(user: &Address, k2: &u32) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[229u8, 213u8, 98u8, 35u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(user, k2)));
            out
        }
        #[inline(always)]
        pub fn encode_user_items_len(user: &Address) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[252u8, 81u8, 7u8, 18u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(user));
            out
        }
        #[inline(always)]
        pub fn encode_user_items_list(
            user: &Address,
            start: &u32,
            size: &u32,
        ) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[51u8, 226u8, 110u8, 199u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(user, start, size)));
            out
        }
        /// High-level call functions
        ///
        /// For each message function using Codec encoding, generates corresponding call_* functions,
        /// providing one-stop interfaces for encoding, calling, and decoding.
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn set_value(
            callee: &Address,
            value: &u32,
        ) -> Result<Result<(), Error>, ReturnErrorCode> {
            let input = encode_set_value(value);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Result<(), Error> as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn get_value(callee: &Address) -> Result<u32, ReturnErrorCode> {
            let input = encode_get_value();
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <u32 as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn get_value_option(
            callee: &Address,
        ) -> Result<Option<u32>, ReturnErrorCode> {
            let input = encode_get_value_option();
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Option<u32> as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn get_cluster(callee: &Address) -> Result<Cluster, ReturnErrorCode> {
            let input = encode_get_cluster();
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Cluster as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn set_cluster(
            callee: &Address,
            cluster: &Cluster,
        ) -> Result<Result<(), Error>, ReturnErrorCode> {
            let input = encode_set_cluster(cluster);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Result<(), Error> as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn set_owner(
            callee: &Address,
            new_owner: &Address,
        ) -> Result<Result<(), Error>, ReturnErrorCode> {
            let input = encode_set_owner(new_owner);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Result<(), Error> as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn get_owner(callee: &Address) -> Result<Address, ReturnErrorCode> {
            let input = encode_get_owner();
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Address as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn set_balance(
            callee: &Address,
            user: &Address,
            balance: &u64,
        ) -> Result<Result<(), Error>, ReturnErrorCode> {
            let input = encode_set_balance(user, balance);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Result<(), Error> as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn get_balance(
            callee: &Address,
            user: &Address,
        ) -> Result<u64, ReturnErrorCode> {
            let input = encode_get_balance(user);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <u64 as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn set_user_info(
            callee: &Address,
            user: &Address,
            info_type: &u8,
            value: &u32,
        ) -> Result<Result<(), Error>, ReturnErrorCode> {
            let input = encode_set_user_info(user, info_type, value);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Result<(), Error> as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn get_user_info(
            callee: &Address,
            user: &Address,
            info_type: &u8,
        ) -> Result<u32, ReturnErrorCode> {
            let input = encode_get_user_info(user, info_type);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <u32 as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn transfer_balance(
            callee: &Address,
            from: &Address,
            to: &Address,
            amount: &u64,
        ) -> Result<Result<(), Error>, ReturnErrorCode> {
            let input = encode_transfer_balance(from, to, amount);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Result<(), Error> as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn records_push(
            callee: &Address,
            value: &u64,
        ) -> Result<Option<u32>, ReturnErrorCode> {
            let input = encode_records_push(value);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Option<u32> as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn records_get(callee: &Address, id: &u32) -> Result<u64, ReturnErrorCode> {
            let input = encode_records_get(id);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <u64 as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn records_len(callee: &Address) -> Result<u32, ReturnErrorCode> {
            let input = encode_records_len();
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <u32 as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn records_list(
            callee: &Address,
            start: &u32,
            size: &u32,
        ) -> Result<Vec<(u32, u64)>, ReturnErrorCode> {
            let input = encode_records_list(start, size);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Vec<(u32, u64)> as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn user_items_push(
            callee: &Address,
            user: &Address,
            value: &u32,
        ) -> Result<Option<u32>, ReturnErrorCode> {
            let input = encode_user_items_push(user, value);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Option<u32> as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn user_items_get(
            callee: &Address,
            user: &Address,
            k2: &u32,
        ) -> Result<u32, ReturnErrorCode> {
            let input = encode_user_items_get(user, k2);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <u32 as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn user_items_len(
            callee: &Address,
            user: &Address,
        ) -> Result<u32, ReturnErrorCode> {
            let input = encode_user_items_len(user);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <u32 as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Call the remote contract's message, returns decoded return value.
        #[inline(always)]
        pub fn user_items_list(
            callee: &Address,
            user: &Address,
            start: &u32,
            size: &u32,
        ) -> Result<Vec<(u32, u32)>, ReturnErrorCode> {
            let input = encode_user_items_list(user, start, size);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Vec<(u32, u32)> as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// Constructor parameter encoding functions
        /// Encode constructor call data: 4-byte selector + SCALE args (matches `deploy()` input after `#[revive_contract]` codegen).
        #[inline(always)]
        pub fn encode_deploy(initial_value: &u32) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::new();
            out.extend_from_slice(&[244u8, 230u8, 84u8, 160u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(initial_value));
            out
        }
        /// Contract instantiation functions
        ///
        /// Used to deploy new contract instances, handling constructor calls and return value decoding
        /// Instantiate contract (call constructor), returns (new contract address, decoded constructor return value).
        ///
        /// Note: `deposit_limit` is the storage deposit limit. Pass `U256::MAX` for "no specific limit" (recommended default),
        /// aligning with pallet-revive examples.
        #[inline(always)]
        pub fn instantiate_deploy(
            __instantiate_code_hash: &wrevive_api::H256,
            initial_value: &u32,
            deposit_limit: &wrevive_api::U256,
            value: &wrevive_api::U256,
        ) -> Result<(wrevive_api::Address, Result<(), Error>), ReturnErrorCode> {
            let input_data = encode_deploy(initial_value);
            let mut addr = [0u8; 20];
            let mut out_buf = [0u8; 256];
            let mut out_slice = out_buf.as_mut_slice();
            let mut cursor = &mut out_slice;
            let __deposit_bytes = deposit_limit.as_bytes(wrevive_api::CallMode::Sol);
            let __value_bytes = value.as_bytes(wrevive_api::CallMode::Sol);
            wrevive_api::env()
                .instantiate(
                    pallet_revive_uapi::CallFlags::empty(),
                    __instantiate_code_hash.as_bytes(),
                    u64::MAX,
                    u64::MAX,
                    &__deposit_bytes,
                    &__value_bytes,
                    &input_data,
                    &mut addr,
                    Some(&mut cursor),
                )?;
            let ret = <Result<
                (),
                Error,
            > as wrevive_api::Decode>::decode(&mut &out_buf[..])
                .map_err(|_| ReturnErrorCode::CalleeTrapped)?;
            Ok((wrevive_api::Address::from(addr), ret))
        }
    }
}
#[allow(unreachable_code)]
/// Contract deployment entry function
///
/// This function is automatically generated by #[revive_contract] macro to:
/// 1. Decode deployment parameters (selector + SCALE encoded constructor arguments)
/// 2. Call user-defined constructor
/// 3. Encode constructor return value and set it to contract return data
///
/// # Parameter Format
/// - input_data: selector(4 bytes) + SCALE encoded constructor parameters
///
/// # Return Values
/// - Success: SCALE encoded constructor return value
/// - Failure: REVERT flag + error information
pub fn deploy() {
    wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
    let __input_len = wrevive_api::env().call_data_size().min(1024) as usize;
    let __input_vec = if __input_len > 0 {
        wrevive_api::env().call_data_copy(0, __input_len)
    } else {
        let empty = ::alloc::vec::Vec::new();
        empty
    };
    let __input: &[u8] = &__input_vec;
    if __input.len() < 4 {
        wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]);
        return;
    }
    let __input = &__input[4..];
    let mut __scale_input = __input;
    let initial_value: u32 = match <u32 as wrevive_api::Decode>::decode(
        &mut __scale_input,
    ) {
        Ok(val) => val,
        Err(_) => {
            wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]);
            return;
        }
    };
    let __ret = contract::deploy(initial_value);
    let __encoded = wrevive_api::Encode::encode(&__ret);
    wrevive_api::env()
        .return_value(
            if let Err(_) = &__ret {
                wrevive_api::ReturnFlags::REVERT
            } else {
                wrevive_api::ReturnFlags::empty()
            },
            &__encoded,
        );
}
#[allow(unreachable_code)]
/// Contract call entry function (message dispatcher)
///
/// This function is automatically generated by #[revive_contract] macro to:
/// 1. Read call data (selector + SCALE encoded arguments)
/// 2. Dispatch to corresponding message function based on selector
/// 3. Decode arguments and call user-defined message function
/// 4. Encode return value and set it to contract return data
///
/// # Parameter Format
/// - input_data: selector(4 bytes) + SCALE encoded message function arguments
///
/// # Selector Dispatch Logic
/// - Each message function has a unique 4-byte selector
/// - Selector is generated using first 4 bytes of Keccak-256 hash of function name
/// - Supports explicit selector specification: `#[revive(message, selector = 0x...)]`
///
/// # Return Values
/// - Success: SCALE encoded message function return value
/// - Failure: REVERT flag + error information
/// - Unknown selector: call fallback function or directly REVERT
pub fn call() {
    let __input_len = wrevive_api::env().call_data_size().min(1024) as usize;
    let __input_vec = if __input_len > 0 {
        wrevive_api::env().call_data_copy(0, __input_len)
    } else {
        let empty = ::alloc::vec::Vec::new();
        empty
    };
    let __input: &[u8] = &__input_vec;
    if __input_len < 4 {
        wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]);
    }
    let __sel = u32::from_be_bytes([__input[0], __input[1], __input[2], __input[3]]);
    match __sel {
        2950139990u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let value: u32 = match <u32 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::set_value(value);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = match &__ret {
                Ok(_) => wrevive_api::ReturnFlags::empty(),
                Err(_) => wrevive_api::ReturnFlags::REVERT,
            };
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        3261600723u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let __ret = contract::get_value();
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        2246435773u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let __ret = contract::get_value_option();
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        4207312817u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Sol);
            let mut __sol_off: usize = 0;
            let value: u32 = match <u32 as pvm_contract_types::SolDecode>::decode_at(
                &__input[4..],
                __sol_off,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            __sol_off
                += if <u32 as pvm_contract_types::SolEncode>::IS_DYNAMIC {
                    32
                } else {
                    pvm_contract_types::SolEncode::encode_len(&value)
                };
            let __ret = contract::set_value_sol(value);
            match &__ret {
                Ok(ok_val) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::empty(), &[]);
                }
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[])
                }
            }
        }
        1848507827u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Sol);
            let __ret = contract::get_value_sol();
            let __len = pvm_contract_types::SolEncode::encode_len(&__ret);
            let mut __buf = ::alloc::vec::from_elem(0u8, __len);
            pvm_contract_types::SolEncode::encode_to(&__ret, &mut __buf);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__buf);
        }
        2232444275u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let __ret = contract::get_cluster();
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        3231458310u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let cluster: Cluster = match <Cluster as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::set_cluster(cluster);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = match &__ret {
                Ok(_) => wrevive_api::ReturnFlags::empty(),
                Err(_) => wrevive_api::ReturnFlags::REVERT,
            };
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        1430221647u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let new_owner: Address = match <Address as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::set_owner(new_owner);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = match &__ret {
                Ok(_) => wrevive_api::ReturnFlags::empty(),
                Err(_) => wrevive_api::ReturnFlags::REVERT,
            };
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        2884504570u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let __ret = contract::get_owner();
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        2342555464u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let user: Address = match <Address as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let balance: u64 = match <u64 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::set_balance(user, balance);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = match &__ret {
                Ok(_) => wrevive_api::ReturnFlags::empty(),
                Err(_) => wrevive_api::ReturnFlags::REVERT,
            };
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        1000214996u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let user: Address = match <Address as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::get_balance(user);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        450572617u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let user: Address = match <Address as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let info_type: u8 = match <u8 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let value: u32 = match <u32 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::set_user_info(user, info_type, value);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = match &__ret {
                Ok(_) => wrevive_api::ReturnFlags::empty(),
                Err(_) => wrevive_api::ReturnFlags::REVERT,
            };
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        3598401391u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let user: Address = match <Address as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let info_type: u8 = match <u8 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::get_user_info(user, info_type);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        3262845453u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let from: Address = match <Address as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let to: Address = match <Address as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let amount: u64 = match <u64 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::transfer_balance(from, to, amount);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = match &__ret {
                Ok(_) => wrevive_api::ReturnFlags::empty(),
                Err(_) => wrevive_api::ReturnFlags::REVERT,
            };
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        3401411403u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let value: u64 = match <u64 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::records_push(value);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        390564210u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let id: u32 = match <u32 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::records_get(id);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        2893505882u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let __ret = contract::records_len();
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        2865914824u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let start: u32 = match <u32 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let size: u32 = match <u32 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::records_list(start, size);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        1699430201u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let user: Address = match <Address as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let value: u32 = match <u32 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::user_items_push(user, value);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        3855966755u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let user: Address = match <Address as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let k2: u32 = match <u32 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::user_items_get(user, k2);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        4233168658u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let user: Address = match <Address as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::user_items_len(user);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        870477511u32 => {
            wrevive_api::env().set_call_mode(wrevive_api::CallMode::Codec);
            let mut __scale_input = &__input[4..];
            let user: Address = match <Address as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let start: u32 = match <u32 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let size: u32 = match <u32 as wrevive_api::Decode>::decode(
                &mut __scale_input,
            ) {
                Ok(val) => val,
                Err(_) => {
                    wrevive_api::env()
                        .return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    return;
                }
            };
            let __ret = contract::user_items_list(user, start, size);
            let __encoded = wrevive_api::Encode::encode(&__ret);
            let __flags = wrevive_api::ReturnFlags::empty();
            wrevive_api::env().return_value(__flags, &__encoded);
        }
        _ => wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]),
    }
}
