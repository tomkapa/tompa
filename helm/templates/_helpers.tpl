{{/*
Expand the name of the chart.
*/}}
{{- define "tompa.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
Truncate at 63 chars because some Kubernetes name fields have a limit.
*/}}
{{- define "tompa.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart label value (name-version).
*/}}
{{- define "tompa.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels applied to all resources.
*/}}
{{- define "tompa.labels" -}}
helm.sh/chart: {{ include "tompa.chart" . }}
{{ include "tompa.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels — used by Deployment and Service to match pods.
*/}}
{{- define "tompa.selectorLabels" -}}
app.kubernetes.io/name: {{ include "tompa.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Name of the secret created by the ExternalSecret.
*/}}
{{- define "tompa.secretName" -}}
{{ include "tompa.fullname" . }}-secrets
{{- end }}

{{/*
Name of the ConfigMap holding non-secret configuration.
*/}}
{{- define "tompa.configMapName" -}}
{{ include "tompa.fullname" . }}-config
{{- end }}
