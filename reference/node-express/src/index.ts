import { createApp } from "./wallet.ts";

const secret = process.env.POLARKAST_SHARED_SECRET ?? process.env.SHARED_SECRET; // SHARED_SECRET: old name, still read
if (!secret) throw new Error("Set POLARKAST_SHARED_SECRET (the per-operator shared secret).");
const port = Number(process.env.PORT ?? 8081);
createApp(secret).listen(port, () => console.log(`wallet listening on :${port}`));
