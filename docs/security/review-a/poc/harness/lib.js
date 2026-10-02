const {
  Connection, Keypair, PublicKey, SystemProgram, Transaction,
  TransactionInstruction, sendAndConfirmTransaction, LAMPORTS_PER_SOL,
} = require('@solana/web3.js');
const {
  createMint, getOrCreateAssociatedTokenAccount, mintTo,
  getAssociatedTokenAddressSync, createAssociatedTokenAccountInstruction,
  getAccount, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID,
} = require('@solana/spl-token');
const crypto = require('crypto');

const RPC = 'http://127.0.0.1:18899';
const MPL_HYBRID = new PublicKey('MPL4o4wMzndgh8T1NVDxELQCj5UQfYTYEkabX3wNKtb');
const MPL_CORE  = new PublicKey('CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d');
const FEE_WALLET_V2 = new PublicKey('C3iyKknpNPeZXQEVLkR8ZJxcgB8xdsqXkyrV1RwEmdrD');
const SLOT_HASHES = new PublicKey('SysvarS1otHashes111111111111111111111111111');

const disc = n => crypto.createHash('sha256').update('global:' + n).digest().slice(0, 8);
const u64 = n => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; };
const u16 = n => { const b = Buffer.alloc(2); b.writeUInt16LE(n); return b; };
const str = s => { const d = Buffer.from(s, 'utf8'); return Buffer.concat([u32(d.length), d]); };
const u32 = n => { const b = Buffer.alloc(4); b.writeUInt32LE(n); return b; };
const optU64 = v => v == null ? Buffer.from([0]) : Buffer.concat([Buffer.from([1]), u64(v)]);
const optU16 = v => v == null ? Buffer.from([0]) : Buffer.concat([Buffer.from([1]), u16(v)]);
const optStr = v => v == null ? Buffer.from([0]) : Buffer.concat([Buffer.from([1]), str(v)]);

const conn = new Connection(RPC, 'confirmed');
async function airdrop(pk, sol=100){ const s = await conn.requestAirdrop(pk, sol*LAMPORTS_PER_SOL); await conn.confirmTransaction(s,'confirmed'); }
async function send(ixs, signers){ const tx=new Transaction().add(...ixs); return await sendAndConfirmTransaction(conn, tx, signers, {commitment:'confirmed'}); }

const m = (pk,s=false,w=false)=>({pubkey:pk,isSigner:s,isWritable:w});

// ---- MPL Core: create collection ----
async function createCollection(payer, updateAuthority, name='C', uri='u'){
  const collection = Keypair.generate();
  const data = Buffer.concat([Buffer.from([1]), str(name), str(uri), Buffer.from([0])]); // disc=1, name,uri, plugins=None
  const keys = [
    m(collection.publicKey,true,true),
    m(updateAuthority.publicKey,false,false), // updateAuthority present -> readonly non-signer (arg form). But we want UA=updateAuthority.
    m(payer.publicKey,true,true),
    m(SystemProgram.programId,false,false),
  ];
  const ix = new TransactionInstruction({programId:MPL_CORE, keys, data});
  await send([ix],[payer, collection]);
  return collection;
}
// ---- MPL Core: create asset in collection, owned by `owner` pubkey ----
async function createAsset(payer, collection, collectionAuthority, owner, name='A', uri='u'){
  const asset = Keypair.generate();
  // disc=0, DataState (AccountState=0), name, uri, plugins=None
  const data = Buffer.concat([Buffer.from([0]), Buffer.from([0]), str(name), str(uri), Buffer.from([0])]);
  const keys = [
    m(asset.publicKey,true,true),                 // asset
    m(collection.publicKey,false,true),           // collection (present -> writable)
    m(collectionAuthority.publicKey,true,false),  // authority (present -> signer, must be collection UA)
    m(payer.publicKey,true,true),                 // payer
    m(owner,false,false),                         // owner (present)
    m(MPL_CORE,false,false),                      // updateAuthority absent -> MPL_CORE placeholder (defaults to collection)
    m(SystemProgram.programId,false,false),       // system
    m(MPL_CORE,false,false),                      // logWrapper absent
  ];
  const ix = new TransactionInstruction({programId:MPL_CORE, keys, data});
  const signers = [payer, asset];
  if(!collectionAuthority.publicKey.equals(payer.publicKey)) signers.push(collectionAuthority);
  await send([ix],signers);
  return asset;
}

function escrowPda(authority){ return PublicKey.findProgramAddressSync([Buffer.from('escrow'), authority.toBuffer()], MPL_HYBRID)[0]; }
function recipePda(collection){ return PublicKey.findProgramAddressSync([Buffer.from('recipe'), collection.toBuffer()], MPL_HYBRID)[0]; }

async function initEscrowV2(authority){
  const escrow = escrowPda(authority.publicKey);
  const data = disc('init_escrow_v2');
  const keys=[m(escrow,false,true), m(authority.publicKey,true,true), m(SystemProgram.programId,false,false)];
  await send([new TransactionInstruction({programId:MPL_HYBRID,keys,data})],[authority]);
  return escrow;
}
async function initRecipeV1(authority, collection, token, feeLocation, opts){
  const recipe = recipePda(collection);
  const feeAta = getAssociatedTokenAddressSync(token, feeLocation, true);
  const ixdata = Buffer.concat([
    disc('init_recipe_v1'),
    str(opts.name||'N'), str(opts.uri||'u'),
    u64(opts.max), u64(opts.min), u64(opts.amount),
    u64(opts.feeCapture||0), u64(opts.feeRelease||0),
    u64(opts.solFeeCapture||0), u64(opts.solFeeRelease||0),
    u16(opts.path||0),
  ]);
  const keys=[
    m(recipe,false,true), m(authority.publicKey,true,true), m(collection,false,false),
    m(token,false,false), m(feeLocation,false,false), m(feeAta,false,true),
    m(SystemProgram.programId,false,false), m(TOKEN_PROGRAM_ID,false,false), m(ASSOCIATED_TOKEN_PROGRAM_ID,false,false),
  ];
  await send([new TransactionInstruction({programId:MPL_HYBRID,keys,data:ixdata})],[authority]);
  return recipe;
}
async function updateRecipeV1(authority, collection, token, feeLocation, fields){
  const recipe = recipePda(collection);
  const data = Buffer.concat([
    disc('update_recipe_v1'),
    optStr(fields.name), optStr(fields.uri),
    optU64(fields.max), optU64(fields.min), optU64(fields.amount),
    optU64(fields.feeCapture), optU64(fields.feeRelease),
    optU64(fields.solFeeCapture), optU64(fields.solFeeRelease),
    optU16(fields.path),
  ]);
  const keys=[
    m(recipe,false,true), m(authority.publicKey,true,true), m(collection,false,true),
    m(token,false,false), m(feeLocation,false,false), m(SystemProgram.programId,false,false),
  ];
  await send([new TransactionInstruction({programId:MPL_HYBRID,keys,data})],[authority]);
}
function captureKeys(owner, authorityPk, recipe, escrow, asset, collection, uAta, eAta, token, feeAta, feeProject){
  return [
    m(owner.publicKey,true,true), m(authorityPk,false,true), m(recipe,false,true), m(escrow,false,true),
    m(asset,false,true), m(collection,false,true), m(uAta,false,true), m(eAta,false,true),
    m(token,false,true), m(feeAta,false,true), m(FEE_WALLET_V2,false,true), m(feeProject,false,true),
    m(SLOT_HASHES,false,false), m(MPL_CORE,false,false), m(SystemProgram.programId,false,false),
    m(TOKEN_PROGRAM_ID,false,false), m(ASSOCIATED_TOKEN_PROGRAM_ID,false,false),
  ];
}
module.exports = {
  conn, airdrop, send, m, disc, u64, PublicKey, Keypair, SystemProgram, TransactionInstruction,
  createMint, getOrCreateAssociatedTokenAccount, mintTo, getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction, getAccount, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID,
  MPL_HYBRID, MPL_CORE, FEE_WALLET_V2, SLOT_HASHES,
  createCollection, createAsset, escrowPda, recipePda, initEscrowV2, initRecipeV1, updateRecipeV1, captureKeys,
};
