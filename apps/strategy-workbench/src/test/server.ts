import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";

import { validStatusPayload } from "./fixtures";

export const server = setupServer(
  http.get("/api/mvp/v1/status", () => HttpResponse.json(validStatusPayload)),
);
