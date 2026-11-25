import { createApp } from "./wallet.ts";

const secret = process.env.SHARED_SECRET;
if (!secret) throw new Error("Set SHARED_SECRET (the per-operator shared secret).");
const port = Number(process.env.PORT ?? 8081);
createApp(secret).listen(port, () => console.log(`wallet listening on :${port}`));
