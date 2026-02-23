{{/*
Common labels
*/}}
{{- define "fractal-flame.labels" -}}
app.kubernetes.io/name: fractal-flame
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Backend selector labels
*/}}
{{- define "fractal-flame.backend.selectorLabels" -}}
app: fractal-flame-backend
{{- end }}

{{/*
Frontend selector labels
*/}}
{{- define "fractal-flame.frontend.selectorLabels" -}}
app: fractal-flame-frontend
{{- end }}
