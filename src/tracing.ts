import { registerInstrumentations } from "@opentelemetry/instrumentation";
import { HttpInstrumentation } from "@opentelemetry/instrumentation-http";
import { ExpressInstrumentation } from "@opentelemetry/instrumentation-express";
import {
  AlwaysOnSampler,
  SimpleSpanProcessor,
} from "@opentelemetry/sdk-trace-base";
import { NodeTracerProvider } from "@opentelemetry/sdk-trace-node";
import { Resource } from "@opentelemetry/resources";
import { OTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-http";

import { SemanticResourceAttributes as ResourceAttributesSC } from "@opentelemetry/semantic-conventions";

export function setupTracing() {
  if (!process.env.OTEL_ENABLED) return;

  console.log("Setting up tracing.");

  const serviceName = process.env.OTEL_SERVICE_NAME;

  const provider = new NodeTracerProvider({
    resource: new Resource({
      [ResourceAttributesSC.SERVICE_NAME]: serviceName,
    }),
    sampler: new AlwaysOnSampler(),
  });

  registerInstrumentations({
    instrumentations: [
      // Express instrumentation expects HTTP layer to be instrumented
      new HttpInstrumentation(),
      new ExpressInstrumentation(),
    ],
  });

  const exporter = new OTLPTraceExporter({
    url: process.env.OTLP_ENDPOINT_URL,
  });

  provider.addSpanProcessor(new SimpleSpanProcessor(exporter));
  provider.register();
}
