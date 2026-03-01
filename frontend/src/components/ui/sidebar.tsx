import * as React from 'react'
import { PanelLeft } from 'lucide-react'
import { cn } from '@/lib/utils'

// Halo Sidebar — matches Sidebar component
// bg-sidebar, rounded-[24px] (radius-m), shadow, border border-sidebar-border
// Header: Logo + toggle button
// Content slot: nav items (vertical, padded)
// Footer: user info + ellipsis

interface SidebarContextValue {
  collapsed: boolean
  setCollapsed: (collapsed: boolean) => void
}

const SidebarContext = React.createContext<SidebarContextValue>({
  collapsed: false,
  setCollapsed: () => {},
})

interface SidebarProviderProps {
  defaultCollapsed?: boolean
  children: React.ReactNode
  className?: string
}

function SidebarProvider({ defaultCollapsed = false, children, className }: SidebarProviderProps) {
  const [collapsed, setCollapsed] = React.useState(defaultCollapsed)
  return (
    <SidebarContext.Provider value={{ collapsed, setCollapsed }}>
      <div className={cn('flex h-full', className)}>{children}</div>
    </SidebarContext.Provider>
  )
}

interface SidebarProps extends React.HTMLAttributes<HTMLDivElement> {
  width?: number
  collapsedWidth?: number
}

function Sidebar({ className, width = 256, collapsedWidth = 72, ...props }: SidebarProps) {
  const { collapsed } = React.useContext(SidebarContext)
  return (
    <aside
      className={cn(
        'flex flex-col bg-sidebar border border-sidebar-border rounded-[24px] shadow-[0_4px_21px_-2px_rgba(16,24,40,0.03)] transition-all duration-300',
        className
      )}
      style={{ width: collapsed ? collapsedWidth : width }}
      {...props}
    />
  )
}

function SidebarHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  const { collapsed, setCollapsed } = React.useContext(SidebarContext)
  return (
    <div className={cn('flex items-center gap-2 p-6', className)} {...props}>
      {!collapsed && props.children}
      <button
        type="button"
        className="ml-auto flex h-8 w-8 items-center justify-center rounded-full text-sidebar-foreground hover:bg-sidebar-accent transition-colors"
        onClick={() => setCollapsed(!collapsed)}
        aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      >
        <PanelLeft className="h-5 w-5" />
      </button>
    </div>
  )
}

function SidebarContent({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div className={cn('flex flex-1 flex-col gap-0 overflow-y-auto px-4 py-0', className)} {...props} />
  )
}

function SidebarFooter({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('flex items-center gap-2 p-6', className)} {...props} />
}

function SidebarSectionTitle({ className, children, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  const { collapsed } = React.useContext(SidebarContext)
  if (collapsed) return null
  return (
    <div className={cn('flex items-center px-2 py-2', className)} {...props}>
      <span className="flex-1 truncate text-sm font-semibold text-sidebar-foreground leading-[1.5]">
        {children}
      </span>
    </div>
  )
}

interface SidebarItemProps extends React.HTMLAttributes<HTMLDivElement> {
  active?: boolean
  icon?: React.ReactNode
  label?: string
}

function SidebarItem({ className, active, icon, label, children, ...props }: SidebarItemProps) {
  const { collapsed } = React.useContext(SidebarContext)
  return (
    <div
      className={cn(
        'flex cursor-pointer items-center gap-2 rounded-[24px] py-3 transition-colors',
        collapsed ? 'px-4 justify-center' : 'px-6',
        active
          ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium'
          : 'text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
        className
      )}
      {...props}
    >
      {icon && <span className="flex h-6 w-6 shrink-0 items-center justify-center">{icon}</span>}
      {!collapsed && (
        <span className="flex-1 truncate text-base leading-[1.5]">{label ?? children}</span>
      )}
    </div>
  )
}

function SidebarGroup({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('flex flex-col', className)} {...props} />
}

export {
  SidebarProvider,
  Sidebar,
  SidebarHeader,
  SidebarContent,
  SidebarFooter,
  SidebarSectionTitle,
  SidebarItem,
  SidebarGroup,
}
