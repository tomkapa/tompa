import * as React from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import mermaid from 'mermaid'
import { cn } from '@/lib/utils'

// ── Mermaid initialisation ────────────────────────────────────────────────────

let mermaidInitialised = false

function initialiseMermaid() {
  if (mermaidInitialised) return
  mermaidInitialised = true
  const style = getComputedStyle(document.documentElement)
  mermaid.initialize({
    startOnLoad: false,
    theme: 'base',
    themeVariables: {
      primaryColor: style.getPropertyValue('--primary').trim() || '#5749F4',
      primaryTextColor: style.getPropertyValue('--primary-foreground').trim() || '#FFFFFF',
      primaryBorderColor: style.getPropertyValue('--border').trim() || '#C5C5CB',
      lineColor: style.getPropertyValue('--muted-foreground').trim() || '#616167',
      secondaryColor: style.getPropertyValue('--muted').trim() || '#F5F5F5',
      tertiaryColor: style.getPropertyValue('--accent').trim() || '#F5F5F5',
      background: style.getPropertyValue('--background').trim() || '#FFFFFF',
      mainBkg: style.getPropertyValue('--card').trim() || '#FFFFFF',
      nodeBorder: style.getPropertyValue('--border').trim() || '#C5C5CB',
      clusterBkg: style.getPropertyValue('--muted').trim() || '#F5F5F5',
      titleColor: style.getPropertyValue('--foreground').trim() || '#2A2933',
      edgeLabelBackground: style.getPropertyValue('--background').trim() || '#FFFFFF',
      fontFamily: 'Inter, system-ui, sans-serif',
    },
  })
}

// ── MermaidBlock ──────────────────────────────────────────────────────────────

interface MermaidBlockProps {
  code: string
}

function MermaidBlock({ code }: MermaidBlockProps) {
  const containerRef = React.useRef<HTMLDivElement>(null)
  const [error, setError] = React.useState(false)

  React.useEffect(() => {
    initialiseMermaid()
    const id = `mermaid-${crypto.randomUUID()}`
    void mermaid
      .render(id, code)
      .then(({ svg }) => {
        if (containerRef.current) {
          containerRef.current.innerHTML = svg
          setError(false)
        }
      })
      .catch((err: unknown) => {
        console.error('[MermaidBlock]', { code }, err)
        setError(true)
      })
  }, [code])

  if (error) {
    return (
      <pre className="overflow-x-auto rounded-lg bg-muted p-3 text-[12px] font-mono text-muted-foreground">
        <code>{code}</code>
      </pre>
    )
  }

  return <div ref={containerRef} className="mermaid-block" />
}

// ── MarkdownViewer ────────────────────────────────────────────────────────────

interface MarkdownViewerProps {
  content: string
  className?: string
}

export function MarkdownViewer({ content, className }: MarkdownViewerProps) {
  return (
    <div className={cn('md-prose', className)}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          code({ className: cls, children, ...rest }) {
            const language = /language-(\w+)/.exec(cls ?? '')?.[1]
            const codeText = String(children).replace(/\n$/, '')
            if (language === 'mermaid') {
              return <MermaidBlock code={codeText} />
            }
            // inline code: no node prop needed
            const isBlock = codeText.includes('\n')
            if (isBlock) {
              return (
                <pre>
                  <code className={cls} {...rest}>
                    {children}
                  </code>
                </pre>
              )
            }
            return (
              <code className={cls} {...rest}>
                {children}
              </code>
            )
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}
