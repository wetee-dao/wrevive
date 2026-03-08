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
use wrevive_api::{Address, Encode, List, List2D, Mapping, Storage, Vec, env};
use wrevive_macro::{list, list_2d, mapping, revive_contract, storage};
pub use datas::*;
use wrevive_api::Decode;
use pvm_contract_types::{SolDecode, SolEncode};
pub mod contract {
    use super::*;
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
    pub fn deploy(initial_value: u32) -> Result<(), Error> {
        VALUE.set(env(), &initial_value);
        OWNER.set(env(), &env().caller());
        env().deposit_event(EMPTY_TOPICS, &initial_value.to_le_bytes().as_slice());
        Ok(())
    }
    pub fn set_value(value: u32) -> Result<(), Error> {
        VALUE.set(env(), &value);
        env().deposit_event(EMPTY_TOPICS, &value.to_le_bytes().as_slice());
        Ok(())
    }
    pub fn get_value() -> u32 {
        VALUE.get(env()).unwrap_or(0)
    }
    pub fn get_value_option() -> Option<u32> {
        VALUE.get(env()).ok()
    }
    /// Set value; only current owner may call (else revert). for solidity.
    pub fn set_value_sol(value: u32) -> Result<(), Error> {
        VALUE.set(env(), &value);
        env().deposit_event(EMPTY_TOPICS, &value.to_le_bytes().as_slice());
        Ok(())
    }
    /// Get value; only current owner may call (else revert). for solidity.
    pub fn get_value_sol() -> u32 {
        VALUE.get(env()).unwrap_or(0)
    }
    pub fn get_cluster() -> Cluster {
        CLUSTER.get(env()).unwrap_or(Cluster::default())
    }
    pub fn set_cluster(cluster: Cluster) -> Result<(), Error> {
        CLUSTER.set(env(), &cluster);
        Ok(())
    }
    /// Set owner; only current owner may call (else revert).
    /// 设置 owner；仅当前 owner 可调用，否则 revert。
    pub fn set_owner(new_owner: Address) -> Result<(), Error> {
        let caller = env().caller();
        let current_owner = get_owner();
        if caller != current_owner {
            return Err(Error::Unauthorized);
        }
        OWNER.set(env(), &new_owner);
        Ok(())
    }
    pub fn get_owner() -> Address {
        OWNER.get(env()).unwrap_or(Address::zero())
    }
    /// 设置用户余额（使用 Mapping）
    pub fn set_balance(user: Address, balance: u64) -> Result<(), Error> {
        BALANCE_MAPPING.set(env(), &user, &balance);
        Ok(())
    }
    /// 获取用户余额（使用 Mapping）
    pub fn get_balance(user: Address) -> u64 {
        BALANCE_MAPPING.get(env(), &user).unwrap_or(0)
    }
    /// 设置用户信息（key = (user, info_type)）
    pub fn set_user_info(user: Address, info_type: u8, value: u32) -> Result<(), Error> {
        USER_INFO_MAPPING.set(env(), &(user, info_type), &value);
        Ok(())
    }
    /// 获取用户信息
    pub fn get_user_info(user: Address, info_type: u8) -> u32 {
        USER_INFO_MAPPING.get(env(), &(user, info_type)).unwrap_or(0)
    }
    /// Transfer balance from one account to another. Only the sender (`from`) may call (else revert).
    /// Reverts if `from` has insufficient balance. Self-transfer (from == to) is a no-op.
    /// 转账：仅 from 可发起；余额不足时 revert；from == to 时不操作。
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
        let from_balance = BALANCE_MAPPING.get(env(), &from).unwrap_or(0);
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }
        let to_balance = BALANCE_MAPPING.get(env(), &to).unwrap_or(0);
        BALANCE_MAPPING.set(env(), &from, &(from_balance - amount));
        BALANCE_MAPPING.set(env(), &to, &(to_balance + amount));
        Ok(())
    }
    /// 向全局 records 列表追加一条 u64，返回分配到的 id
    pub fn records_push(value: u64) -> Option<u32> {
        RECORDS.insert(env(), &value)
    }
    /// 按 id 取 records 中的值
    pub fn records_get(id: u32) -> u64 {
        RECORDS.get(env(), &id).unwrap_or(0)
    }
    /// 全局 records 长度
    pub fn records_len() -> u32 {
        RECORDS.len(env())
    }
    /// 分页：从 start 起取最多 size 条 (id, value)。返回长度 0 表示无数据或参数不合法
    pub fn records_list(start: u32, size: u32) -> Vec<(u32, u64)> {
        RECORDS.list(env(), start, size)
    }
    /// 在指定用户下追加一条 u32，返回该用户下的 k2
    pub fn user_items_push(user: Address, value: u32) -> Option<u32> {
        USER_ITEMS.insert(env(), &user, &value)
    }
    /// 取用户 user 下第 k2 条
    pub fn user_items_get(user: Address, k2: u32) -> u32 {
        USER_ITEMS.get(env(), &user, k2).unwrap_or(0)
    }
    /// 用户 user 下的条目数量
    pub fn user_items_len(user: Address) -> u32 {
        USER_ITEMS.len(env(), &user)
    }
    /// 分页：用户 user 下从 start 起取最多 size 条 (k2, value)
    pub fn user_items_list(user: Address, start: u32, size: u32) -> Vec<(u32, u32)> {
        USER_ITEMS.list(env(), &user, start, size)
    }
    /// 合约间调用接口：selector 常量、encode_*、call_raw、call_*、constructor 的 encode_* / instantiate_*。
    pub mod api {
        use super::*;
        use wrevive_api::*;
        pub const SELECTOR_SET_VALUE: [u8; 4] = [42u8, 191u8, 193u8, 98u8];
        pub const SELECTOR_GET_VALUE: [u8; 4] = [250u8, 196u8, 46u8, 228u8];
        pub const SELECTOR_GET_VALUE_OPTION: [u8; 4] = [153u8, 77u8, 40u8, 115u8];
        pub const SELECTOR_SET_VALUE_SOL: [u8; 4] = [214u8, 32u8, 246u8, 142u8];
        pub const SELECTOR_GET_VALUE_SOL: [u8; 4] = [81u8, 196u8, 170u8, 77u8];
        pub const SELECTOR_GET_CLUSTER: [u8; 4] = [6u8, 181u8, 211u8, 133u8];
        pub const SELECTOR_SET_CLUSTER: [u8; 4] = [219u8, 187u8, 215u8, 8u8];
        pub const SELECTOR_SET_OWNER: [u8; 4] = [48u8, 199u8, 36u8, 15u8];
        pub const SELECTOR_GET_OWNER: [u8; 4] = [207u8, 137u8, 86u8, 83u8];
        pub const SELECTOR_SET_BALANCE: [u8; 4] = [185u8, 5u8, 240u8, 118u8];
        pub const SELECTOR_GET_BALANCE: [u8; 4] = [177u8, 6u8, 182u8, 251u8];
        pub const SELECTOR_SET_USER_INFO: [u8; 4] = [32u8, 151u8, 92u8, 111u8];
        pub const SELECTOR_GET_USER_INFO: [u8; 4] = [148u8, 233u8, 37u8, 59u8];
        pub const SELECTOR_TRANSFER_BALANCE: [u8; 4] = [162u8, 224u8, 122u8, 104u8];
        pub const SELECTOR_RECORDS_PUSH: [u8; 4] = [26u8, 77u8, 125u8, 164u8];
        pub const SELECTOR_RECORDS_GET: [u8; 4] = [203u8, 234u8, 201u8, 127u8];
        pub const SELECTOR_RECORDS_LEN: [u8; 4] = [144u8, 209u8, 249u8, 22u8];
        pub const SELECTOR_RECORDS_LIST: [u8; 4] = [101u8, 150u8, 99u8, 168u8];
        pub const SELECTOR_USER_ITEMS_PUSH: [u8; 4] = [166u8, 210u8, 235u8, 63u8];
        pub const SELECTOR_USER_ITEMS_GET: [u8; 4] = [118u8, 109u8, 134u8, 50u8];
        pub const SELECTOR_USER_ITEMS_LEN: [u8; 4] = [26u8, 95u8, 146u8, 66u8];
        pub const SELECTOR_USER_ITEMS_LIST: [u8; 4] = [233u8, 127u8, 22u8, 239u8];
        /// 调用指定合约，传入已编码的 input（4 字节 selector + 编码参数），返回合约返回的原始字节。
        #[inline(always)]
        pub fn call_raw(
            callee: &Address,
            input: &[u8],
        ) -> Result<Vec<u8>, ReturnErrorCode> {
            let r = wrevive_api::env()
                .call(
                    pallet_revive_uapi::CallFlags::empty(),
                    callee,
                    10_000_000,
                    10_000_000,
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
        #[inline(always)]
        pub fn encode_set_value(value: &u32) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[42u8, 191u8, 193u8, 98u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(value));
            out
        }
        #[inline(always)]
        pub fn encode_get_value() -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4);
            out.extend_from_slice(&[250u8, 196u8, 46u8, 228u8]);
            out
        }
        #[inline(always)]
        pub fn encode_get_value_option() -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4);
            out.extend_from_slice(&[153u8, 77u8, 40u8, 115u8]);
            out
        }
        #[inline(always)]
        pub fn encode_get_cluster() -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4);
            out.extend_from_slice(&[6u8, 181u8, 211u8, 133u8]);
            out
        }
        #[inline(always)]
        pub fn encode_set_cluster(cluster: &Cluster) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[219u8, 187u8, 215u8, 8u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(cluster));
            out
        }
        #[inline(always)]
        pub fn encode_set_owner(new_owner: &Address) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[48u8, 199u8, 36u8, 15u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(new_owner));
            out
        }
        #[inline(always)]
        pub fn encode_get_owner() -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4);
            out.extend_from_slice(&[207u8, 137u8, 86u8, 83u8]);
            out
        }
        #[inline(always)]
        pub fn encode_set_balance(user: &Address, balance: &u64) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[185u8, 5u8, 240u8, 118u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(user, balance)));
            out
        }
        #[inline(always)]
        pub fn encode_get_balance(user: &Address) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[177u8, 6u8, 182u8, 251u8]);
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
            out.extend_from_slice(&[32u8, 151u8, 92u8, 111u8]);
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
            out.extend_from_slice(&[148u8, 233u8, 37u8, 59u8]);
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
            out.extend_from_slice(&[162u8, 224u8, 122u8, 104u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(from, to, amount)));
            out
        }
        #[inline(always)]
        pub fn encode_records_push(value: &u64) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[26u8, 77u8, 125u8, 164u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(value));
            out
        }
        #[inline(always)]
        pub fn encode_records_get(id: &u32) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[203u8, 234u8, 201u8, 127u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(id));
            out
        }
        #[inline(always)]
        pub fn encode_records_len() -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4);
            out.extend_from_slice(&[144u8, 209u8, 249u8, 22u8]);
            out
        }
        #[inline(always)]
        pub fn encode_records_list(start: &u32, size: &u32) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[101u8, 150u8, 99u8, 168u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(start, size)));
            out
        }
        #[inline(always)]
        pub fn encode_user_items_push(
            user: &Address,
            value: &u32,
        ) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[166u8, 210u8, 235u8, 63u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(user, value)));
            out
        }
        #[inline(always)]
        pub fn encode_user_items_get(user: &Address, k2: &u32) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 64);
            out.extend_from_slice(&[118u8, 109u8, 134u8, 50u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(user, k2)));
            out
        }
        #[inline(always)]
        pub fn encode_user_items_len(user: &Address) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(4 + 32);
            out.extend_from_slice(&[26u8, 95u8, 146u8, 66u8]);
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
            out.extend_from_slice(&[233u8, 127u8, 22u8, 239u8]);
            out.extend_from_slice(&wrevive_api::Encode::encode(&(user, start, size)));
            out
        }
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
        #[inline(always)]
        pub fn get_value(callee: &Address) -> Result<u32, ReturnErrorCode> {
            let input = encode_get_value();
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <u32 as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
        #[inline(always)]
        pub fn get_cluster(callee: &Address) -> Result<Cluster, ReturnErrorCode> {
            let input = encode_get_cluster();
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Cluster as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
        #[inline(always)]
        pub fn get_owner(callee: &Address) -> Result<Address, ReturnErrorCode> {
            let input = encode_get_owner();
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <Address as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
        #[inline(always)]
        pub fn records_get(callee: &Address, id: &u32) -> Result<u64, ReturnErrorCode> {
            let input = encode_records_get(id);
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <u64 as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// 调用远端合约的该 message，返回已解码的返回值。
        #[inline(always)]
        pub fn records_len(callee: &Address) -> Result<u32, ReturnErrorCode> {
            let input = encode_records_len();
            let raw = call_raw(callee, &input)?;
            let mut cur = &raw[..];
            <u32 as wrevive_api::Decode>::decode(&mut cur)
                .map_err(|_| ReturnErrorCode::CalleeTrapped)
        }
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 调用远端合约的该 message，返回已解码的返回值。
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
        /// 编码 constructor 参数（无 selector），用于 env::instantiate 的 input_data。
        #[inline(always)]
        pub fn encode_deploy(initial_value: &u32) -> alloc::vec::Vec<u8> {
            wrevive_api::Encode::encode(initial_value)
        }
        /// 实例化合约（调用 constructor），返回 (新合约地址, 已解码的 constructor 返回值)。
        #[inline(always)]
        pub fn instantiate_deploy(
            __instantiate_code_hash: &wrevive_api::H256,
            initial_value: &u32,
            value: &wrevive_api::U256,
            deposit: &wrevive_api::U256,
        ) -> Result<(wrevive_api::Address, Result<(), Error>), ReturnErrorCode> {
            let input_data = encode_deploy(initial_value);
            let mut addr = [0u8; 20];
            let mut out_buf = [0u8; 256];
            let mut out_slice = out_buf.as_mut_slice();
            let mut cursor = &mut out_slice;
            wrevive_api::env()
                .instantiate(
                    pallet_revive_uapi::CallFlags::empty(),
                    __instantiate_code_hash.as_bytes(),
                    10_000_000,
                    10_000_000,
                    deposit.as_bytes(),
                    value.as_bytes(),
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
pub fn deploy() {
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
pub fn call() {
    let __input_len = wrevive_api::env().call_data_size().min(1024) as usize;
    let __input_vec = if __input_len > 0 {
        wrevive_api::env().call_data_copy(0, __input_len)
    } else {
        let empty = ::alloc::vec::Vec::new();
        empty
    };
    let __input: &[u8] = &__input_vec;
    if __input_len >= 4 {
        let __sel = u32::from_be_bytes([__input[0], __input[1], __input[2], __input[3]]);
        match __sel {
            717209954u32 => {
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
            4207161060u32 => {
                let __ret = contract::get_value();
                let __encoded = wrevive_api::Encode::encode(&__ret);
                let __flags = wrevive_api::ReturnFlags::empty();
                wrevive_api::env().return_value(__flags, &__encoded);
            }
            2571970675u32 => {
                let __ret = contract::get_value_option();
                let __encoded = wrevive_api::Encode::encode(&__ret);
                let __flags = wrevive_api::ReturnFlags::empty();
                wrevive_api::env().return_value(__flags, &__encoded);
            }
            3592484494u32 => {
                let mut __sol_off: usize = 0;
                let value: u32 = <u32 as pvm_contract_types::SolDecode>::decode_at(
                    &__input[4..],
                    __sol_off,
                );
                __sol_off += pvm_contract_types::SolEncode::encode_len(&value);
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
            1371843149u32 => {
                let __ret = contract::get_value_sol();
                let __len = pvm_contract_types::SolEncode::encode_len(&__ret);
                let mut __buf = ::alloc::vec::from_elem(0u8, __len);
                pvm_contract_types::SolEncode::encode_to(&__ret, &mut __buf);
                let __flags = wrevive_api::ReturnFlags::empty();
                wrevive_api::env().return_value(__flags, &__buf);
            }
            112579461u32 => {
                let __ret = contract::get_cluster();
                let __encoded = wrevive_api::Encode::encode(&__ret);
                let __flags = wrevive_api::ReturnFlags::empty();
                wrevive_api::env().return_value(__flags, &__encoded);
            }
            3686520584u32 => {
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
            818357263u32 => {
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
            3481884243u32 => {
                let __ret = contract::get_owner();
                let __encoded = wrevive_api::Encode::encode(&__ret);
                let __flags = wrevive_api::ReturnFlags::empty();
                wrevive_api::env().return_value(__flags, &__encoded);
            }
            3104174198u32 => {
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
            2970007291u32 => {
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
            546790511u32 => {
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
            2498307387u32 => {
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
            2732620392u32 => {
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
            441286052u32 => {
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
            3421161855u32 => {
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
            2429679894u32 => {
                let __ret = contract::records_len();
                let __encoded = wrevive_api::Encode::encode(&__ret);
                let __flags = wrevive_api::ReturnFlags::empty();
                wrevive_api::env().return_value(__flags, &__encoded);
            }
            1704354728u32 => {
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
            2798840639u32 => {
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
            1986889266u32 => {
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
            442470978u32 => {
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
            3917420271u32 => {
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
    } else {
        wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]);
    }
}
