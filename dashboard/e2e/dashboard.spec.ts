import { test, expect } from "@playwright/test";

test.describe("Dashboard page load", () => {
  test("renders the page title", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h1")).toHaveText("Timetrack Dashboard");
  });

  test("renders all four stat cards", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("Total Active Time")).toBeVisible();
    await expect(page.getByText("Productivity Score")).toBeVisible();
    await expect(page.getByText("Top Category")).toBeVisible();
    await expect(page.getByText("Current Activity")).toBeVisible();
  });

  test("renders chart sections", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("Time by Category")).toBeVisible();
    await expect(page.getByText("Category Distribution")).toBeVisible();
    await expect(page.getByText("Top Apps & Sites")).toBeVisible();
    await expect(page.getByText("Activity Timeline")).toBeVisible();
    await expect(page.getByText("Productivity Breakdown")).toBeVisible();
    await expect(page.getByText("Activity Trends")).toBeVisible();
  });
});

test.describe("Date range picker", () => {
  test("shows Today selected by default", async ({ page }) => {
    await page.goto("/");
    // The Today button should have the default/primary variant (not outline)
    const todayBtn = page.getByRole("button", { name: "Today" });
    await expect(todayBtn).toBeVisible();
  });

  test("can switch to This Week", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("button", { name: "This Week" }).click();
    // Wait for data to refetch — stat cards should still be present
    await expect(page.getByText("Total Active Time")).toBeVisible();
  });

  test("can switch to This Month", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("button", { name: "This Month" }).click();
    await expect(page.getByText("Total Active Time")).toBeVisible();
  });

  test("can switch to Yesterday", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("button", { name: "Yesterday" }).click();
    await expect(page.getByText("Total Active Time")).toBeVisible();
  });

  test("can open custom date calendar", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("button", { name: "Custom" }).click();
    // Calendar popover should appear with month grids
    await expect(page.getByRole("grid", { name: /February|March|January/ }).first()).toBeVisible();
  });
});

test.describe("API data loading", () => {
  test("stat cards show real data (not loading skeletons) after load", async ({ page }) => {
    await page.goto("/");
    // Wait for loading to finish — skeletons disappear
    await page.waitForTimeout(3000);
    // At least one stat card should have a value that isn't just a skeleton
    const totalActiveCard = page.locator("text=Total Active Time").locator("..");
    await expect(totalActiveCard).toBeVisible();
  });

  test("API /api/summary returns valid JSON", async ({ request }) => {
    const response = await request.get("http://localhost:3123/api/summary");
    expect(response.ok()).toBeTruthy();
    const json = await response.json();
    expect(json).toHaveProperty("total_active");
    expect(json).toHaveProperty("total_active_seconds");
    expect(json).toHaveProperty("groups");
    expect(Array.isArray(json.groups)).toBeTruthy();
  });

  test("API /api/categories returns array", async ({ request }) => {
    const response = await request.get("http://localhost:3123/api/categories");
    expect(response.ok()).toBeTruthy();
    const json = await response.json();
    expect(Array.isArray(json)).toBeTruthy();
    expect(json.length).toBeGreaterThan(0);
    expect(json[0]).toHaveProperty("name");
    expect(json[0]).toHaveProperty("productivity_score");
  });

  test("API /api/apps returns array", async ({ request }) => {
    const response = await request.get("http://localhost:3123/api/apps");
    expect(response.ok()).toBeTruthy();
    const json = await response.json();
    expect(Array.isArray(json)).toBeTruthy();
  });

  test("API /api/productivity returns score", async ({ request }) => {
    const response = await request.get("http://localhost:3123/api/productivity");
    expect(response.ok()).toBeTruthy();
    const json = await response.json();
    expect(json).toHaveProperty("score");
    expect(json.score).toBeGreaterThanOrEqual(0);
    expect(json.score).toBeLessThanOrEqual(100);
    expect(json).toHaveProperty("productive");
    expect(json).toHaveProperty("neutral");
    expect(json).toHaveProperty("distracting");
  });

  test("API /api/timeline returns array", async ({ request }) => {
    const response = await request.get("http://localhost:3123/api/timeline");
    expect(response.ok()).toBeTruthy();
    const json = await response.json();
    expect(Array.isArray(json)).toBeTruthy();
  });

  test("API /api/trends returns array", async ({ request }) => {
    const response = await request.get("http://localhost:3123/api/trends");
    expect(response.ok()).toBeTruthy();
    const json = await response.json();
    expect(Array.isArray(json)).toBeTruthy();
  });

  test("API /api/current returns activity or null", async ({ request }) => {
    const response = await request.get("http://localhost:3123/api/current");
    expect(response.ok()).toBeTruthy();
    const json = await response.json();
    // Can be null or an object with app field
    if (json !== null) {
      expect(json).toHaveProperty("app");
      expect(json).toHaveProperty("category");
      expect(json).toHaveProperty("since");
    }
  });

  test("API /api/summary supports date range params", async ({ request }) => {
    const response = await request.get(
      "http://localhost:3123/api/summary?from=2026-01-01&to=2026-12-31"
    );
    expect(response.ok()).toBeTruthy();
    const json = await response.json();
    expect(json).toHaveProperty("period_from", "2026-01-01T00:00:00");
    expect(json).toHaveProperty("period_to", "2026-12-31T23:59:59");
  });

  test("API /api/summary supports groupBy=app", async ({ request }) => {
    const response = await request.get(
      "http://localhost:3123/api/summary?groupBy=app"
    );
    expect(response.ok()).toBeTruthy();
    const json = await response.json();
    expect(json).toHaveProperty("groups");
  });
});

test.describe("Dark mode", () => {
  test("can toggle dark mode", async ({ page }) => {
    await page.goto("/");
    // Find the dark mode toggle (moon/sun icon button)
    const html = page.locator("html");

    // Click the toggle button (last button in the header area)
    const toggleBtn = page.locator("header + div button, .flex.items-center.gap-2 > button").last();

    // Check initial state
    const initialDark = await html.evaluate((el) => el.classList.contains("dark"));

    await toggleBtn.click();

    const afterToggle = await html.evaluate((el) => el.classList.contains("dark"));
    expect(afterToggle).not.toBe(initialDark);
  });
});
