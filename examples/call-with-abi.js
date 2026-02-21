/**
 * 使用 ABI 调用 wetee_contract（pallet-revive / PolkaVM）
 * 需安装: npm i @polkadot/api
 *
 * ABI 为 ink! 风格：spec.messages[].label 为方法名，.selector 为 4 字节 hex，
 * .args[].type.displayName 为参数类型，.returnType 为 null 或 { displayName: ["u32"] } 等。
 * 调用约定: payload = selector (4 bytes) + 参数（u32 小端，AccountId 20 字节）
 * 链上 pallet 名称可能是 revive 或 contracts，请按实际链替换 api.tx.revive
 */

const { ApiPromise, WsProvider } = require('@polkadot/api');

// 与 abi/contract.json 中的 selectors 一致
const SELECTORS = {
  set: new Uint8Array([0x60, 0xfe, 0x47, 0xb1]),
  get: new Uint8Array([0x6d, 0x4c, 0xe6, 0x33]),
  setOwner: new Uint8Array([0x13, 0xaf, 0x40, 0x35]),
  getOwner: new Uint8Array([0x8f, 0x8f, 0x9f, 0x8f]),
};

function encodeSet(value) {
  const buf = new Uint8Array(4 + 4);
  buf.set(SELECTORS.set, 0);
  new DataView(buf.buffer).setUint32(4, value, true);
  return buf;
}

function encodeGet() {
  return SELECTORS.get;
}

function encodeSetOwner(ownerBytes20) {
  if (ownerBytes20.length !== 20) throw new Error('owner 必须为 20 字节');
  const buf = new Uint8Array(4 + 20);
  buf.set(SELECTORS.setOwner, 0);
  buf.set(ownerBytes20, 4);
  return buf;
}

function encodeGetOwner() {
  return SELECTORS.getOwner;
}

function decodeU32(data) {
  if (data.length < 4) return 0;
  return new DataView(data.buffer, data.byteOffset, 4).getUint32(0, true);
}

function decodeAccountId20(data) {
  if (data.length < 20) return new Uint8Array(20);
  return data.slice(0, 20);
}

async function main() {
  const provider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({ provider });

  const contractAddress = 'CONTRACT_ACCOUNT_ID'; // 实例化后的合约地址
  const caller = 'CALLER_SS58'; // 调用者

  // 示例 1: 调用 set(42)
  const setPayload = encodeSet(42);
  const txSet = api.tx.revive.call(
    contractAddress,
    0,        // value
    1000000,  // gas_limit 按链配置调整
    null,     // storage_deposit_limit
    setPayload
  );
  // await txSet.signAndSend(caller, (res) => { ... });

  // 示例 2: 只读 get()（用 dryRun 或 call，视链提供的 API 而定）
  const getPayload = encodeGet();
  // 若链提供 api.call.revive.call / api.call.contractsApi.call，可在此做只读调用
  // 否则通过 api.tx.revive.call + 提交交易后从事件/result 取 returnData 解码
  // 解码: decodeU32(returnData)

  // 示例 3: getOwner()
  const getOwnerPayload = encodeGetOwner();
  // 同上，用 call 或 api.tx.revive.call 后解析 output
  const ownerBytes = decodeAccountId20(new Uint8Array(20)); // 占位，实际从 result 取
  console.log('getOwner() =>', Buffer.from(ownerBytes).toString('hex'));

  await provider.disconnect();
}

main().catch(console.error);
