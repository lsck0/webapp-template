import { IndexRouteComponent } from "@/routes/index";
import { test, expect } from "vitest";
import { render, screen } from "@testing-library/react";

test("Sample UI Test", () => {
    render(<IndexRouteComponent />);

    expect(screen.getByText(/hello world/i).tagName.toLowerCase()).toBe("p");
    expect(screen.getByText(/hello world/i).tagName.toLowerCase()).not.toBe("div");
});
