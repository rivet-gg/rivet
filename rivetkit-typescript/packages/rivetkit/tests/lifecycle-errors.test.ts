import { describe, expect, test } from "vitest";
import { ActorError } from "../src/client/errors";
import {
	isRetryableLifecycleRequestError,
	retryOnLifecycleBoundary,
} from "../src/client/lifecycle-errors";

describe("lifecycle error retry classification", () => {
	test.each([
		"service_unavailable",
		"actor_wake_retries_exceeded",
		"actor_stopped_while_waiting",
		"tunnel_request_aborted",
		"tunnel_message_timeout",
		"tunnel_response_closed",
		"gateway_response_start_timeout",
	])("classifies guard.%s as retryable", (code) => {
		expect(
			isRetryableLifecycleRequestError(
				new ActorError("guard", code, "transient gateway error"),
			),
		).toBe(true);
	});

	test("does not retry unconfirmed request delivery", () => {
		expect(
			isRetryableLifecycleRequestError(
				new ActorError(
					"guard",
					"request_delivery_unconfirmed",
					"Actor did not respond to request start",
				),
			),
		).toBe(false);
	});
});

test("retries a stale-generation rejection before application admission", async () => {
	let attempts = 0;
	const value = await retryOnLifecycleBoundary(
		async () => {
			if (++attempts === 1)
				throw new ActorError(
					"envoy",
					"actor_generation_mismatch",
					"Actor generation does not match",
				);
			return "accepted";
		},
		{ initialDelayMs: 1 },
	);
	expect(value).toBe("accepted");
	expect(attempts).toBe(2);
	expect(
		isRetryableLifecycleRequestError(
			new ActorError(
				"envoy",
				"request_delivery_unconfirmed",
				"Unknown delivery",
			),
		),
	).toBe(false);
});
