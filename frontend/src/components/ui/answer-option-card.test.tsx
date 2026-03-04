import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { AnswerOptionCard } from './answer-option-card'

const defaultProps = {
  label: 'Option A',
  pros: 'Fast to implement',
  cons: 'Less flexible',
  selected: false,
  recommended: false,
  disabled: false,
  onSelect: vi.fn(),
}

describe('AnswerOptionCard', () => {
  it('renders label correctly', () => {
    render(<AnswerOptionCard {...defaultProps} />)
    expect(screen.getByText('Option A')).toBeInTheDocument()
  })

  it('calls onSelect when radio circle clicked', () => {
    const onSelect = vi.fn()
    render(<AnswerOptionCard {...defaultProps} onSelect={onSelect} />)
    fireEvent.click(screen.getByLabelText('Select Option A'))
    expect(onSelect).toHaveBeenCalledTimes(1)
  })

  it('clicking label text expands instead of selecting', () => {
    const onSelect = vi.fn()
    render(<AnswerOptionCard {...defaultProps} onSelect={onSelect} />)
    fireEvent.click(screen.getByText('Option A'))
    expect(onSelect).not.toHaveBeenCalled()
    expect(screen.getByText('Fast to implement')).toBeInTheDocument()
  })

  it('disabled state prevents click', () => {
    const onSelect = vi.fn()
    render(<AnswerOptionCard {...defaultProps} disabled={true} onSelect={onSelect} />)
    fireEvent.click(screen.getByText('Option A'))
    expect(onSelect).not.toHaveBeenCalled()
  })

  it('selected state applies primary bg styling', () => {
    render(<AnswerOptionCard {...defaultProps} selected={true} />)
    const button = screen.getByRole('button', { name: 'Option A' })
    expect(button.className).toContain('bg-primary')
  })

  it('default state applies background styling', () => {
    render(<AnswerOptionCard {...defaultProps} />)
    expect(screen.getByText('Option A')).toBeInTheDocument()
  })

  it('shows AI suggested badge when recommended', () => {
    render(<AnswerOptionCard {...defaultProps} recommended={true} />)
    expect(screen.getByText('AI suggested')).toBeInTheDocument()
  })

  it('does not show AI suggested badge when not recommended', () => {
    render(<AnswerOptionCard {...defaultProps} recommended={false} />)
    expect(screen.queryByText('AI suggested')).not.toBeInTheDocument()
  })

  it('expands to show pros/cons when card body clicked', () => {
    render(<AnswerOptionCard {...defaultProps} />)
    expect(screen.queryByText('Fast to implement')).not.toBeInTheDocument()
    fireEvent.click(screen.getByText('Option A'))
    expect(screen.getByText('Fast to implement')).toBeInTheDocument()
    expect(screen.getByText('Less flexible')).toBeInTheDocument()
  })

  it('collapses when card body clicked again', () => {
    render(<AnswerOptionCard {...defaultProps} />)
    fireEvent.click(screen.getByText('Option A'))
    expect(screen.getByText('Fast to implement')).toBeInTheDocument()
    fireEvent.click(screen.getByText('Option A'))
    expect(screen.queryByText('Fast to implement')).not.toBeInTheDocument()
  })

  it('selected state does not show chevron or pros/cons', () => {
    render(<AnswerOptionCard {...defaultProps} selected={true} />)
    expect(screen.queryByLabelText('Select Option A')).not.toBeInTheDocument()
    expect(screen.queryByText('Fast to implement')).not.toBeInTheDocument()
  })
})
