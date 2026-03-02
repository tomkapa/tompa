import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { TabSwitcher } from './tab-switcher'

const tabs = [
  { id: 'tab1', label: 'Overview' },
  { id: 'tab2', label: 'Details' },
  { id: 'tab3', label: 'History' },
]

describe('TabSwitcher', () => {
  it('renders all tab labels', () => {
    render(<TabSwitcher tabs={tabs} activeId="tab1" onChange={vi.fn()} />)
    expect(screen.getByText('Overview')).toBeInTheDocument()
    expect(screen.getByText('Details')).toBeInTheDocument()
    expect(screen.getByText('History')).toBeInTheDocument()
  })

  it('active tab has distinct styling', () => {
    render(<TabSwitcher tabs={tabs} activeId="tab2" onChange={vi.fn()} />)
    const activeButton = screen.getByText('Details')
    expect(activeButton.className).toContain('bg-card')
  })

  it('inactive tab does not have active styling', () => {
    render(<TabSwitcher tabs={tabs} activeId="tab1" onChange={vi.fn()} />)
    const inactiveButton = screen.getByText('Details')
    expect(inactiveButton.className).not.toContain('bg-card')
  })

  it('clicking a tab calls onChange with correct id', () => {
    const onChange = vi.fn()
    render(<TabSwitcher tabs={tabs} activeId="tab1" onChange={onChange} />)
    fireEvent.click(screen.getByText('Details'))
    expect(onChange).toHaveBeenCalledWith('tab2')
  })

  it('clicking active tab still calls onChange', () => {
    const onChange = vi.fn()
    render(<TabSwitcher tabs={tabs} activeId="tab1" onChange={onChange} />)
    fireEvent.click(screen.getByText('Overview'))
    expect(onChange).toHaveBeenCalledWith('tab1')
  })
})
