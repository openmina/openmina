{{/*
Liveness probe: openmina node is alive when its statemachine progresses.
*/}}
{{- define "openmina.livenessProbe" }}
{{- if .Values.probes.liveness }}
livenessProbe:
  initialDelaySeconds: 20
  periodSeconds: 10
  failureThreshold: 60
  httpGet:
    path: /healthz
    port: 3000
{{- end }}
{{- end }}

{{/*
Readiness probe: openmina node considered ready when it is in sync with the network.
*/}}
{{- define "openmina.readinessProbe" }}
{{- if .Values.probes.readiness }}
readinessProbe:
  initialDelaySeconds: 60
  periodSeconds: 20
  failureThreshold: 60
  httpGet:
    path: /readyz
    port: 3000
{{- end }}
{{- end }}
