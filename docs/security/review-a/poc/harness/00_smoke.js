const L = require('./lib.js');
(async()=>{
  const payer = L.Keypair.generate();
  await L.airdrop(payer.publicKey, 100);
  const creator = payer; // collection update authority + recipe authority
  const col = await L.createCollection(payer, creator, 'Mintmark', 'https://x/');
  console.log('collection', col.publicKey.toBase58());
  const asset = await L.createAsset(payer, col, creator, /*owner*/ L.escrowPda(creator.publicKey));
  console.log('asset (owned by escrow pda)', asset.publicKey.toBase58());
  const escrow = await L.initEscrowV2(creator);
  console.log('escrow', escrow.toBase58());
  console.log('SMOKE OK');
})().catch(e=>{console.error('SMOKE FAIL', e.message); process.exit(1);});
