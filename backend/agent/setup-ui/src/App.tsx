import { useState, useEffect, FormEvent } from 'react'
import './App.css'

type Mode = 'project' | 'dev' | 'standalone'
type Tab = 'config' | 'status'

interface ConfigForm {
  server_url: string
  api_key: string
  github_repo_url: string
  github_access_token: string
  mode: Mode
}

interface StatusData {
  connected: boolean
  last_heartbeat: number | null
  mode: string
}

const DEFAULT_CONFIG: ConfigForm = {
  server_url: '',
  api_key: '',
  github_repo_url: '',
  github_access_token: '',
  mode: 'standalone',
}

export default function App() {
  const [tab, setTab] = useState<Tab>('config')
  const [config, setConfig] = useState<ConfigForm>(DEFAULT_CONFIG)
  const [status, setStatus] = useState<StatusData | null>(null)
  const [saving, setSaving] = useState(false)
  const [saveMsg, setSaveMsg] = useState<{ ok: boolean; text: string } | null>(null)

  useEffect(() => {
    if (tab !== 'status') return
    const load = () =>
      fetch('/api/status')
        .then(r => r.json())
        .then((d: StatusData) => setStatus(d))
        .catch(() => setStatus(null))
    load()
    const id = setInterval(load, 3000)
    return () => clearInterval(id)
  }, [tab])

  const handleSave = async (e: FormEvent) => {
    e.preventDefault()
    setSaving(true)
    setSaveMsg(null)
    try {
      const body = {
        mode: config.mode,
        server_url: config.server_url,
        api_key: config.api_key,
        ...(config.github_repo_url ? { github_repo_url: config.github_repo_url } : {}),
        ...(config.github_access_token ? { github_access_token: config.github_access_token } : {}),
      }
      const res = await fetch('/api/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })
      if (res.ok) {
        setSaveMsg({ ok: true, text: 'Configuration saved. Restart the agent to apply changes.' })
      } else {
        setSaveMsg({ ok: false, text: 'Failed to save configuration.' })
      }
    } catch {
      setSaveMsg({ ok: false, text: 'Network error. Is the agent running?' })
    } finally {
      setSaving(false)
    }
  }

  const formatHeartbeat = (ts: number | null) => {
    if (ts === null) return 'Never'
    return new Date(ts * 1000).toLocaleTimeString()
  }

  return (
    <div className="app">
      <header className="header">
        <div className="header-inner">
          <div className="brand">
            <div className="brand-icon">A</div>
            <span className="brand-name">Agent Setup</span>
          </div>
          <nav className="tabs">
            <button
              className={`tab ${tab === 'config' ? 'active' : ''}`}
              onClick={() => setTab('config')}
            >
              Configuration
            </button>
            <button
              className={`tab ${tab === 'status' ? 'active' : ''}`}
              onClick={() => setTab('status')}
            >
              Status
            </button>
          </nav>
        </div>
      </header>

      <main className="main">
        {tab === 'config' ? (
          <form className="card" onSubmit={handleSave}>
            <div className="card-section">
              <h2 className="section-title">Server Connection</h2>
              <div className="field">
                <label htmlFor="server_url">Server URL</label>
                <input
                  id="server_url"
                  type="text"
                  placeholder="wss://app.yourdomain.com"
                  value={config.server_url}
                  onChange={e => setConfig(c => ({ ...c, server_url: e.target.value }))}
                  required
                />
              </div>
              <div className="field">
                <label htmlFor="api_key">Container API Key</label>
                <input
                  id="api_key"
                  type="password"
                  placeholder="cpk_..."
                  value={config.api_key}
                  onChange={e => setConfig(c => ({ ...c, api_key: e.target.value }))}
                  required
                />
              </div>
            </div>

            <div className="card-section">
              <h2 className="section-title">Repository</h2>
              <div className="field">
                <label htmlFor="github_repo_url">GitHub Repository URL</label>
                <input
                  id="github_repo_url"
                  type="text"
                  placeholder="https://github.com/org/repo"
                  value={config.github_repo_url}
                  onChange={e => setConfig(c => ({ ...c, github_repo_url: e.target.value }))}
                />
              </div>
              <div className="field">
                <label htmlFor="github_access_token">GitHub Access Token</label>
                <input
                  id="github_access_token"
                  type="password"
                  placeholder="ghp_..."
                  value={config.github_access_token}
                  onChange={e => setConfig(c => ({ ...c, github_access_token: e.target.value }))}
                />
              </div>
            </div>

            <div className="card-section">
              <h2 className="section-title">Operating Mode</h2>
              <div className="radio-group">
                {(['project', 'dev', 'standalone'] as Mode[]).map(m => (
                  <label key={m} className="radio-option">
                    <input
                      type="radio"
                      name="mode"
                      value={m}
                      checked={config.mode === m}
                      onChange={() => setConfig(c => ({ ...c, mode: m }))}
                    />
                    <div className="radio-content">
                      <span className="radio-label">{m.charAt(0).toUpperCase() + m.slice(1)}</span>
                      <span className="radio-desc">{MODE_DESCRIPTIONS[m]}</span>
                    </div>
                  </label>
                ))}
              </div>
            </div>

            {saveMsg && (
              <div className={`msg ${saveMsg.ok ? 'msg-ok' : 'msg-err'}`}>
                {saveMsg.text}
              </div>
            )}

            <div className="card-footer">
              <button className="btn-primary" type="submit" disabled={saving}>
                {saving ? 'Saving…' : 'Save & Connect'}
              </button>
            </div>
          </form>
        ) : (
          <div className="card status-card">
            {status ? (
              <div className="status-list">
                <div className="status-item">
                  <span className="status-label">Connection</span>
                  <span className={`badge ${status.connected ? 'badge-ok' : 'badge-err'}`}>
                    {status.connected ? 'Connected' : 'Disconnected'}
                  </span>
                </div>
                <div className="status-item">
                  <span className="status-label">Last Heartbeat</span>
                  <span className="status-value">{formatHeartbeat(status.last_heartbeat)}</span>
                </div>
                <div className="status-item">
                  <span className="status-label">Mode</span>
                  <span className="status-value capitalize">{status.mode}</span>
                </div>
              </div>
            ) : (
              <p className="loading">Loading status…</p>
            )}
            <div className="card-footer">
              <button className="btn-ghost" onClick={() => setTab('config')}>
                Edit Configuration
              </button>
            </div>
          </div>
        )}
      </main>
    </div>
  )
}

const MODE_DESCRIPTIONS: Record<Mode, string> = {
  project: 'Q&A and planning only — no code execution',
  dev: 'Code execution and git operations — no setup UI',
  standalone: 'Full pipeline: Q&A, planning, and code execution',
}
