import * as React from 'react'
import ReactDOM from 'react-dom'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import mermaid from 'mermaid'
import { Maximize2, Minimize2 } from 'lucide-react'
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
  const [svg, setSvg] = React.useState<string | null>(null)
  const [error, setError] = React.useState(false)
  const [expanded, setExpanded] = React.useState(false)

  React.useEffect(() => {
    initialiseMermaid()
    const id = `mermaid-${crypto.randomUUID()}`
    void mermaid
      .render(id, code)
      .then(({ svg: rendered }) => {
        setSvg(rendered)
        setError(false)
      })
      .catch((err: unknown) => {
        console.error('[MermaidBlock]', { code }, err)
        setError(true)
      })
  }, [code])

  // zoom/pan state for expanded modal
  const [scale, setScale] = React.useState(1)
  const [offset, setOffset] = React.useState({ x: 0, y: 0 })
  const dragRef = React.useRef<{ startX: number; startY: number; ox: number; oy: number } | null>(null)
  const canvasRef = React.useRef<HTMLDivElement>(null)
  const svgWrapRef = React.useRef<HTMLDivElement>(null)

  // compute fit-to-screen scale after the modal renders the SVG
  const fitToScreen = React.useCallback(() => {
    const canvas = canvasRef.current
    const wrap = svgWrapRef.current
    if (!canvas || !wrap) return
    const svgEl = wrap.querySelector('svg')
    if (!svgEl) return
    const svgW = svgEl.getBoundingClientRect().width
    const svgH = svgEl.getBoundingClientRect().height
    const { width: cW, height: cH } = canvas.getBoundingClientRect()
    const padding = 48
    const fit = Math.min((cW - padding) / svgW, (cH - padding) / svgH)
    setScale(fit)
    setOffset({ x: 0, y: 0 })
  }, [])

  React.useEffect(() => {
    if (!expanded) return
    // fit after paint so the SVG has dimensions
    const raf = requestAnimationFrame(fitToScreen)

    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') setExpanded(false)
    }
    document.addEventListener('keydown', onKey)
    return () => {
      cancelAnimationFrame(raf)
      document.removeEventListener('keydown', onKey)
    }
  }, [expanded, fitToScreen])

  function onWheel(e: React.WheelEvent) {
    e.preventDefault()
    setScale(s => Math.min(10, Math.max(0.2, s - e.deltaY * 0.001)))
  }

  function onMouseDown(e: React.MouseEvent) {
    dragRef.current = { startX: e.clientX, startY: e.clientY, ox: offset.x, oy: offset.y }
  }

  function onMouseMove(e: React.MouseEvent) {
    if (!dragRef.current) return
    const dx = e.clientX - dragRef.current.startX
    const dy = e.clientY - dragRef.current.startY
    setOffset({ x: dragRef.current.ox + dx, y: dragRef.current.oy + dy })
  }

  function onMouseUp() {
    dragRef.current = null
  }

  if (error) {
    return (
      <pre className="overflow-x-auto rounded-lg bg-muted p-3 text-[12px] font-mono text-muted-foreground">
        <code>{code}</code>
      </pre>
    )
  }

  return (
    <>
      {expanded &&
        ReactDOM.createPortal(
          <div className="fixed inset-0 z-[60] flex flex-col bg-background/95 backdrop-blur-sm animate-in fade-in-0">
            <div className="flex shrink-0 items-center justify-between border-b border-border px-6 py-3">
              <span className="text-sm font-semibold text-foreground">
                Diagram
                <span className="ml-3 text-xs font-normal text-muted-foreground">scroll to zoom · drag to pan</span>
              </span>
              <button
                type="button"
                title="Exit fullscreen (Esc)"
                onClick={() => setExpanded(false)}
                className="flex h-7 w-7 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
              >
                <Minimize2 className="h-4 w-4" />
              </button>
            </div>
            {/* zoom/pan canvas */}
            <div
              ref={canvasRef}
              className="relative flex-1 overflow-hidden cursor-grab active:cursor-grabbing select-none"
              onWheel={onWheel}
              onMouseDown={onMouseDown}
              onMouseMove={onMouseMove}
              onMouseUp={onMouseUp}
              onMouseLeave={onMouseUp}
            >
              <div
                ref={svgWrapRef}
                style={{
                  transform: `translate(calc(-50% + ${offset.x}px), calc(-50% + ${offset.y}px)) scale(${scale})`,
                  position: 'absolute',
                  top: '50%',
                  left: '50%',
                  transformOrigin: 'center center',
                }}
              >
                {svg && (
                  <div
                    // eslint-disable-next-line react/no-danger
                    dangerouslySetInnerHTML={{ __html: svg }}
                  />
                )}
              </div>
              {/* fit-to-screen reset */}
              <button
                type="button"
                title="Reset zoom"
                onClick={fitToScreen}
                className="absolute bottom-4 right-4 rounded border border-border bg-background px-2 py-1 text-xs text-muted-foreground shadow-sm transition-colors hover:bg-accent hover:text-foreground"
              >
                Fit
              </button>
            </div>
          </div>,
          document.body,
        )}

      <div className="group relative">
        {svg && (
          // eslint-disable-next-line react/no-danger
          <div dangerouslySetInnerHTML={{ __html: svg }} className="mermaid-block" />
        )}
        {svg && (
          <button
            type="button"
            title="Expand diagram"
            onClick={() => setExpanded(true)}
            className="absolute right-1 top-1 flex h-6 w-6 items-center justify-center rounded text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100 hover:bg-accent hover:text-foreground"
          >
            <Maximize2 className="h-3 w-3" />
          </button>
        )}
      </div>
    </>
  )
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

// ── ExpandableMarkdownViewer ──────────────────────────────────────────────────
// Read-only viewer with a fullscreen expand button.

interface ExpandableMarkdownViewerProps {
  content: string
  label?: string
  className?: string
}

export function ExpandableMarkdownViewer({ content, label, className }: ExpandableMarkdownViewerProps) {
  const [fullscreen, setFullscreen] = React.useState(false)

  React.useEffect(() => {
    if (!fullscreen) return
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') setFullscreen(false)
    }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [fullscreen])

  return (
    <>
      {fullscreen &&
        ReactDOM.createPortal(
          <div className="fixed inset-0 z-[60] flex flex-col bg-background/95 backdrop-blur-sm animate-in fade-in-0">
            <div className="flex shrink-0 items-center justify-between border-b border-border px-6 py-3">
              <span className="text-sm font-semibold text-foreground">{label ?? 'Description'}</span>
              <button
                type="button"
                title="Exit fullscreen (Esc)"
                onClick={() => setFullscreen(false)}
                className="flex h-7 w-7 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
              >
                <Minimize2 className="h-4 w-4" />
              </button>
            </div>
            <div className="flex-1 overflow-y-auto">
              <div className="mx-auto w-full max-w-2xl px-8 py-10">
                <MarkdownViewer content={content} />
              </div>
            </div>
          </div>,
          document.body,
        )}

      <div className={cn('group relative', className)}>
        <MarkdownViewer content={content} />
        <button
          type="button"
          title="Fullscreen"
          onClick={() => setFullscreen(true)}
          className="absolute right-0 top-0 flex h-6 w-6 items-center justify-center rounded text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100 hover:bg-accent hover:text-foreground"
        >
          <Maximize2 className="h-3 w-3" />
        </button>
      </div>
    </>
  )
}
