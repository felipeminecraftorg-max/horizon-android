// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui.screens;

import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/** A screen: lane 3 runs it on the server's side and Horizon draws what it draws. */
public abstract class Screen implements net.minecraft.client.gui.components.events.GuiEventListener {
    protected final Component title;
    public int width;
    public int height;
    protected net.minecraft.client.Minecraft minecraft;
    protected net.minecraft.client.gui.Font font = new net.minecraft.client.gui.Font();
    private final List<net.minecraft.client.gui.components.Renderable> renderables = new ArrayList<>();
    private final List<net.minecraft.client.gui.components.events.GuiEventListener> children = new ArrayList<>();

    protected Screen(Component title) {
        this.title = title;
    }

    public Component getTitle() {
        return title;
    }

    public final void init(net.minecraft.client.Minecraft minecraft, int width, int height) {
        this.minecraft = minecraft;
        this.width = width;
        this.height = height;
        renderables.clear();
        children.clear();
        init();
    }

    protected void init() {}

    public void resize(net.minecraft.client.Minecraft minecraft, int width, int height) {
        init(minecraft, width, height);
    }

    protected <T extends net.minecraft.client.gui.components.events.GuiEventListener & net.minecraft.client.gui.components.Renderable
            & net.minecraft.client.gui.narration.NarratableEntry> T addRenderableWidget(T widget) {
        renderables.add(widget);
        children.add(widget);
        return widget;
    }

    protected <T extends net.minecraft.client.gui.components.Renderable> T addRenderableOnly(T widget) {
        renderables.add(widget);
        return widget;
    }

    protected <T extends net.minecraft.client.gui.components.events.GuiEventListener & net.minecraft.client.gui.narration.NarratableEntry> T addWidget(
            T widget) {
        children.add(widget);
        return widget;
    }

    protected void removeWidget(net.minecraft.client.gui.components.events.GuiEventListener widget) {
        renderables.remove(widget);
        children.remove(widget);
    }

    protected void clearWidgets() {
        renderables.clear();
        children.clear();
    }

    public List<? extends net.minecraft.client.gui.components.events.GuiEventListener> children() {
        return children;
    }

    public void render(GuiGraphics graphics, int mouseX, int mouseY, float partialTick) {
        renderBackground(graphics, mouseX, mouseY, partialTick);
        for (net.minecraft.client.gui.components.Renderable r : new ArrayList<>(renderables)) r.render(graphics, mouseX, mouseY, partialTick);
    }

    /** Horizon dims the world behind every screen itself. */
    public void renderBackground(GuiGraphics graphics, int mouseX, int mouseY, float partialTick) {}

    public void renderTransparentBackground(GuiGraphics graphics) {}

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        for (net.minecraft.client.gui.components.events.GuiEventListener child : new ArrayList<>(children)) {
            if (child.mouseClicked(mouseX, mouseY, button)) return true;
        }
        return false;
    }

    @Override
    public boolean keyPressed(int key, int scan, int modifiers) {
        if (key == 256 && shouldCloseOnEsc()) {
            onClose();
            return true;
        }
        return false;
    }

    public boolean shouldCloseOnEsc() {
        return true;
    }

    public void onClose() {
        if (minecraft != null) minecraft.setScreen(null);
    }

    public void removed() {}

    public void tick() {}

    public boolean isPauseScreen() {
        return false;
    }
}
