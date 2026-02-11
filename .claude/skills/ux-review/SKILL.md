---
name: ux-review
description: Run a UX audit on the Timely dashboard web application
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash, WebFetch, Task
argument-hint: [component or page]
---

# UX Review

You are a senior frontend UX researcher auditing the Timely dashboard.

Run a comprehensive UX review of the dashboard application. If `$ARGUMENTS` specifies a component or area, focus there. Otherwise audit the full dashboard.

## Review Process

1. **Read the frontend source** — understand component structure, data flow, and rendering logic
   - Dashboard entry: `dashboard/src/App.tsx`
   - Components: `dashboard/src/components/`
   - Hooks: `dashboard/src/hooks/`
   - API layer: `dashboard/src/lib/api.ts`
   - Types: `dashboard/src/lib/types.ts`

2. **Use Playwright MCP** (if available) to open `http://localhost:8080` and take accessibility snapshots of the live dashboard

3. **Evaluate against these UX heuristics:**

### Information Architecture
- Is the most important data (current activity, productivity score) immediately visible?
- Does the data hierarchy match user priorities?
- Are categories, apps, and sites clearly distinguished?
- Is time data presented in human-friendly formats?

### Data Freshness & Feedback
- Does the dashboard auto-refresh? At what interval?
- Is there visual indication of when data was last updated?
- Are loading states clear and non-jarring?
- Does stale data get communicated to the user?

### Visual Design & Clarity
- Are charts readable at a glance?
- Is color usage consistent and meaningful?
- Does the timeline visualization communicate activity patterns effectively?
- Is there visual clutter or information overload?

### Interaction Design
- Can users drill down from summary to detail?
- Is the date range picker intuitive?
- Can users quickly find what they did at a specific time?
- Are categorization rules easy to manage?

### Accessibility
- Color contrast ratios (WCAG AA minimum)
- Screen reader compatibility
- Keyboard navigation
- Responsive layout at different viewport sizes

### Privacy & Sensitivity
- Is sensitive content (NSFW sites, personal data) handled appropriately?
- Can users toggle privacy mode?
- Are explicit page titles hidden by default or on demand?

### Performance
- Are API calls efficient (no redundant fetches)?
- Do components avoid unnecessary re-renders?
- Is the data payload size reasonable?

4. **Produce a prioritized report:**
   - **Critical** — issues that make the dashboard misleading or unusable
   - **High** — significant UX gaps that hurt daily usability
   - **Medium** — improvements that would meaningfully improve the experience
   - **Low** — polish and nice-to-haves

For each finding, include:
- What the issue is
- Where in the code it lives (file:line)
- A concrete fix recommendation
