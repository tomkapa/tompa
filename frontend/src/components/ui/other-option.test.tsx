import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { OtherOption } from './other-option'

describe('OtherOption', () => {
  it('shows "Other" text when not selected', () => {
    render(
      <OtherOption
        selected={false}
        disabled={false}
        value=""
        onChange={vi.fn()}
        onSelect={vi.fn()}
        onSubmit={vi.fn()}
      />
    )
    expect(screen.getByText('Other')).toBeInTheDocument()
  })

  it('clicking the "Other" button calls onSelect', () => {
    const onSelect = vi.fn()
    render(
      <OtherOption
        selected={false}
        disabled={false}
        value=""
        onChange={vi.fn()}
        onSelect={onSelect}
        onSubmit={vi.fn()}
      />
    )
    fireEvent.click(screen.getByRole('button', { name: /Other/i }))
    expect(onSelect).toHaveBeenCalledTimes(1)
  })

  it('shows textarea when selected', () => {
    render(
      <OtherOption
        selected={true}
        disabled={false}
        value=""
        onChange={vi.fn()}
        onSelect={vi.fn()}
        onSubmit={vi.fn()}
      />
    )
    expect(screen.getByPlaceholderText('Describe your approach...')).toBeInTheDocument()
  })

  it('typing in textarea calls onChange with the value', () => {
    const onChange = vi.fn()
    render(
      <OtherOption
        selected={true}
        disabled={false}
        value=""
        onChange={onChange}
        onSelect={vi.fn()}
        onSubmit={vi.fn()}
      />
    )
    const textarea = screen.getByPlaceholderText('Describe your approach...')
    fireEvent.change(textarea, { target: { value: 'My custom approach' } })
    expect(onChange).toHaveBeenCalledWith('My custom approach')
  })

  it('submit calls onSubmit when value is set', () => {
    const onSubmit = vi.fn()
    render(
      <OtherOption
        selected={true}
        disabled={false}
        value="My custom approach"
        onChange={vi.fn()}
        onSelect={vi.fn()}
        onSubmit={onSubmit}
      />
    )
    // Submit button is not disabled when value is present
    const buttons = screen.getAllByRole('button')
    const submitBtn = buttons[buttons.length - 1]
    fireEvent.click(submitBtn)
    expect(onSubmit).toHaveBeenCalledTimes(1)
  })

  it('disabled state prevents click', () => {
    const onSelect = vi.fn()
    render(
      <OtherOption
        selected={false}
        disabled={true}
        value=""
        onChange={vi.fn()}
        onSelect={onSelect}
        onSubmit={vi.fn()}
      />
    )
    const button = screen.getByRole('button')
    expect(button).toBeDisabled()
  })
})
