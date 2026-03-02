import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { StoryTableRow, type StoryRowData } from './story-table-row'

const baseStory: StoryRowData = {
  id: 'story-1',
  title: 'Add user authentication',
  storyType: 'feature',
  status: 'todo',
  ownerName: 'Alice',
  needsAttention: false,
}

describe('StoryTableRow', () => {
  it('renders story title', () => {
    render(<StoryTableRow story={baseStory} onClick={vi.fn()} />)
    expect(screen.getByText('Add user authentication')).toBeInTheDocument()
  })

  it('calls onClick when clicked', () => {
    const onClick = vi.fn()
    render(<StoryTableRow story={baseStory} onClick={onClick} />)
    fireEvent.click(screen.getByRole('row'))
    expect(onClick).toHaveBeenCalledTimes(1)
  })

  it('bug type shows "BUG" tag', () => {
    const bugStory: StoryRowData = { ...baseStory, storyType: 'bug' }
    render(<StoryTableRow story={bugStory} onClick={vi.fn()} />)
    expect(screen.getByText('BUG')).toBeInTheDocument()
  })

  it('done status has opacity styling', () => {
    const doneStory: StoryRowData = { ...baseStory, status: 'done' }
    render(<StoryTableRow story={doneStory} onClick={vi.fn()} />)
    const row = screen.getByRole('row')
    expect(row.className).toContain('opacity-50')
  })

  it('non-done status does not have opacity styling', () => {
    render(<StoryTableRow story={baseStory} onClick={vi.fn()} />)
    const row = screen.getByRole('row')
    expect(row.className).not.toContain('opacity-50')
  })

  it('renders owner name', () => {
    render(<StoryTableRow story={baseStory} onClick={vi.fn()} />)
    expect(screen.getByText('Alice')).toBeInTheDocument()
  })
})
