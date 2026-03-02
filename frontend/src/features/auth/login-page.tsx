import { Layers, Github, Chrome, Brain, GitBranch, MessageCircle } from 'lucide-react'

const FEATURES = [
  { icon: Brain, text: 'AI-driven story decomposition' },
  { icon: GitBranch, text: 'Smart task management' },
  { icon: MessageCircle, text: 'Interactive Q&A workflows' },
] as const

function handleOAuthLogin(provider: 'github' | 'google') {
  window.location.href = `/api/v1/auth/login/${provider}`
}

async function handleDevLogin() {
  await fetch('/api/v1/auth/dev-login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({ email: 'dev@localhost', display_name: 'Dev User' }),
  })
  window.location.href = '/'
}

export function LoginPage() {
  return (
    <div className="flex h-screen bg-background">
      {/* Left Panel — Branding */}
      <div className="hidden lg:flex w-[640px] shrink-0 flex-col items-center justify-center gap-8 bg-primary p-12">
        {/* Logo */}
        <div className="flex flex-col items-center gap-4">
          <div className="flex items-center gap-3">
            <Layers className="h-9 w-9 text-primary-foreground" />
            <span className="text-[32px] font-bold leading-tight text-primary-foreground">
              Tompa
            </span>
          </div>
          <p className="text-base text-primary-foreground/80 text-center">
            AI-Powered Project Management
          </p>
        </div>

        {/* Illustration placeholder */}
        <div className="flex h-[300px] w-[400px] items-center justify-center overflow-hidden rounded-3xl bg-primary-foreground/10">
          <div className="flex flex-col items-center gap-3 text-primary-foreground/40">
            <Layers className="h-16 w-16" />
            <span className="text-sm font-medium">Intelligent Workflows</span>
          </div>
        </div>

        {/* Feature list */}
        <div className="flex flex-col gap-4 w-[360px]">
          {FEATURES.map(({ icon: Icon, text }) => (
            <div key={text} className="flex items-center gap-3">
              <Icon className="h-5 w-5 shrink-0 text-primary-foreground/80" />
              <span className="text-sm text-primary-foreground/80">{text}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Right Panel — Login Form */}
      <div className="flex flex-1 flex-col items-center justify-center gap-6 p-8 md:p-12">
        {/* Mobile logo (shown when left panel is hidden) */}
        <div className="flex lg:hidden items-center gap-3 mb-4">
          <Layers className="h-8 w-8 text-primary" />
          <span className="text-2xl font-bold text-foreground">Tompa</span>
        </div>

        {/* Login Card */}
        <div className="w-full max-w-[420px] overflow-hidden rounded-[40px] border border-border bg-card shadow-lg">
          {/* Card Header */}
          <div className="flex flex-col gap-2 px-8 pt-8">
            <h1 className="text-2xl font-semibold text-card-foreground">
              Welcome back
            </h1>
            <p className="text-sm text-muted-foreground">
              Sign in to your account to continue
            </p>
          </div>

          {/* Card Content — OAuth Buttons */}
          <div className="flex flex-col gap-4 px-8 py-6 pb-8">
            <div className="flex flex-col gap-3">
              <button
                type="button"
                onClick={() => handleOAuthLogin('github')}
                className="flex w-full items-center justify-center gap-2 rounded-full border border-border px-4 py-3.5 text-sm font-medium text-foreground transition-colors hover:bg-accent"
              >
                <Github className="h-4 w-4" />
                GitHub
              </button>
              <button
                type="button"
                onClick={() => handleOAuthLogin('google')}
                className="flex w-full items-center justify-center gap-2 rounded-full border border-border px-4 py-3.5 text-sm font-medium text-foreground transition-colors hover:bg-accent"
              >
                <Chrome className="h-4 w-4" />
                Google
              </button>
            </div>

            {import.meta.env.DEV && (
              <button
                type="button"
                onClick={handleDevLogin}
                className="flex w-full items-center justify-center gap-2 rounded-full border-2 border-dashed border-muted-foreground/30 px-4 py-3.5 text-sm font-medium text-muted-foreground transition-colors hover:border-muted-foreground/50 hover:text-foreground"
              >
                Dev Login
              </button>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center gap-1.5">
          <span className="text-[13px] text-muted-foreground">
            Don't have an account?
          </span>
          <span className="text-[13px] font-semibold text-primary cursor-default">
            Sign up
          </span>
        </div>
      </div>
    </div>
  )
}
