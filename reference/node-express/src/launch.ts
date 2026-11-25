import { createHmac } from "node:crypto";

export type LaunchRequest = {
  session_token: string; player_id: string; game_id: string; currency: string; language: string;
};

// What an operator does when a player opens a game: signed POST /v1/launch, then send the player to game_url.
export async function launch(studioUrl: string, operatorId: string, secret: string, req: LaunchRequest): Promise<string> {
  const body = JSON.stringify(req); // sign exactly the bytes we send
  const res = await fetch(new URL("/v1/launch", studioUrl), {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Operator-Id": operatorId,
      "X-Signature": createHmac("sha256", secret).update(body).digest("hex"),
    },
    body,
  });
  const json = (await res.json()) as { game_url?: string; error?: { code: string; message: string } };
  if (!res.ok || !json.game_url) throw new Error(`launch failed: ${res.status} ${json.error?.code} ${json.error?.message}`);
  return json.game_url;
}
