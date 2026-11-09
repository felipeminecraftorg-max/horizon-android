// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui.components;

import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/** A widget at x, y of a size: hovered while the mouse is on it, clicked when it is pressed there. */
public abstract class AbstractWidget implements Renderable, net.minecraft.client.gui.components.events.GuiEventListener,
        net.minecraft.client.gui.narration.NarratableEntry {
    protected int width;
    protected int height;
    private int x;
    private int y;
    private Component message;
    protected boolean isHovered;
    public boolean active = true;
    public boolean visible = true;
    private boolean focused;
    private Object tooltip;

    public AbstractWidget(int x, int y, int width, int height, Component message) {
        this.x = x;
        this.y = y;
        this.width = width;
        this.height = height;
        this.message = message;
    }

    @Override
    public final void render(GuiGraphics graphics, int mouseX, int mouseY, float partialTick) {
        if (!visible) return;
        isHovered = mouseX >= x && mouseY >= y && mouseX < x + width && mouseY < y + height;
        renderWidget(graphics, mouseX, mouseY, partialTick);
    }

    protected abstract void renderWidget(GuiGraphics graphics, int mouseX, int mouseY, float partialTick);

    public void onClick(double mouseX, double mouseY) {}

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (!active || !visible || button != 0 || !clicked(mouseX, mouseY)) return false;
        onClick(mouseX, mouseY);
        return true;
    }

    protected boolean clicked(double mouseX, double mouseY) {
        return mouseX >= x && mouseY >= y && mouseX < x + width && mouseY < y + height;
    }

    @Override
    public boolean isMouseOver(double mouseX, double mouseY) {
        return visible && clicked(mouseX, mouseY);
    }

    public int getX() {
        return x;
    }

    public int getY() {
        return y;
    }

    public void setX(int x) {
        this.x = x;
    }

    public void setY(int y) {
        this.y = y;
    }

    public void setPosition(int x, int y) {
        this.x = x;
        this.y = y;
    }

    public int getWidth() {
        return width;
    }

    public int getHeight() {
        return height;
    }

    public void setWidth(int width) {
        this.width = width;
    }

    public void setHeight(int height) {
        this.height = height;
    }

    public Component getMessage() {
        return message;
    }

    public void setMessage(Component message) {
        this.message = message;
    }

    public boolean isActive() {
        return visible && active;
    }

    public boolean isHovered() {
        return isHovered;
    }

    public boolean isHoveredOrFocused() {
        return isHovered || focused;
    }

    @Override
    public void setFocused(boolean focused) {
        this.focused = focused;
    }

    @Override
    public boolean isFocused() {
        return focused;
    }

    public void setTooltip(Object tooltip) {
        this.tooltip = tooltip;
    }
}
