import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { AnswerOptionCard } from './answer-option-card'

describe('AnswerOptionCard', () => {
  it('renders text correctly', () => {
    render(
      <AnswerOptionCard text="Option A" selected={false} disabled={false} onSelect={vi.fn()} />
    )
    expect(screen.getByText('Option A')).toBeInTheDocument()
  })

  it('calls onSelect when clicked', () => {
    const onSelect = vi.fn()
    render(
      <AnswerOptionCard text="Option A" selected={false} disabled={false} onSelect={onSelect} />
    )
    fireEvent.click(screen.getByRole('button', { name: 'Option A' }))
    expect(onSelect).toHaveBeenCalledTimes(1)
  })

  it('disabled state prevents click', () => {
    const onSelect = vi.fn()
    render(
      <AnswerOptionCard text="Option A" selected={false} disabled={true} onSelect={onSelect} />
    )
    const button = screen.getByRole('button', { name: 'Option A' })
    expect(button).toBeDisabled()
    fireEvent.click(button)
    expect(onSelect).not.toHaveBeenCalled()
  })

  it('selected state applies distinct styling', () => {
    render(
      <AnswerOptionCard text="Option A" selected={true} disabled={false} onSelect={vi.fn()} />
    )
    const button = screen.getByRole('button', { name: 'Option A' })
    expect(button.className).toContain('bg-primary')
  })

  it('unselected state applies background styling', () => {
    render(
      <AnswerOptionCard text="Option A" selected={false} disabled={false} onSelect={vi.fn()} />
    )
    const button = screen.getByRole('button', { name: 'Option A' })
    expect(button.className).toContain('bg-background')
  })
})
