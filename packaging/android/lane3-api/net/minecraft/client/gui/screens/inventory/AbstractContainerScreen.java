// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui.screens.inventory;

import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/** A menu's screen: its window in the middle, its background, its labels, its slots. */
public abstract class AbstractContainerScreen<T extends net.minecraft.world.inventory.AbstractContainerMenu>
        extends net.minecraft.client.gui.screens.Screen implements MenuAccess<T> {
    protected int imageWidth = 176;
    protected int imageHeight = 166;
    protected int titleLabelX;
    protected int titleLabelY;
    protected int inventoryLabelX;
    protected int inventoryLabelY;
    protected final T menu;
    protected final Component playerInventoryTitle;
    protected int leftPos;
    protected int topPos;

    public AbstractContainerScreen(T menu, net.minecraft.world.entity.player.Inventory inventory, Component title) {
        super(title);
        this.menu = menu;
        this.playerInventoryTitle = Component.translatable("container.inventory");
        this.titleLabelX = 8;
        this.titleLabelY = 6;
        this.inventoryLabelX = 8;
        this.inventoryLabelY = imageHeight - 94;
    }

    @Override
    protected void init() {
        leftPos = (width - imageWidth) / 2;
        topPos = (height - imageHeight) / 2;
    }

    @Override
    public void render(GuiGraphics graphics, int mouseX, int mouseY, float partialTick) {
        renderBg(graphics, partialTick, mouseX, mouseY);
        super.render(graphics, mouseX, mouseY, partialTick);
        graphics.pose().pushPose();
        graphics.pose().translate((float) leftPos, (float) topPos, 0.0F);
        renderLabels(graphics, mouseX, mouseY);
        graphics.pose().popPose();
    }

    protected abstract void renderBg(GuiGraphics graphics, float partialTick, int mouseX, int mouseY);

    protected void renderLabels(GuiGraphics graphics, int mouseX, int mouseY) {
        graphics.drawString(font, title, titleLabelX, titleLabelY, 0x404040, false);
        graphics.drawString(font, playerInventoryTitle, inventoryLabelX, inventoryLabelY, 0x404040, false);
    }

    protected void renderTooltip(GuiGraphics graphics, int mouseX, int mouseY) {}

    @Override
    public boolean keyPressed(int key, int scan, int modifiers) {
        if (key == 256) {
            onClose();
            return true;
        }
        return super.keyPressed(key, scan, modifiers);
    }

    @Override
    public void onClose() {
        if (minecraft != null && minecraft.player != null) minecraft.player.closeContainer();
        super.onClose();
    }

    @Override
    public T getMenu() {
        return menu;
    }

    public int getGuiLeft() {
        return leftPos;
    }

    public int getGuiTop() {
        return topPos;
    }

    public int getXSize() {
        return imageWidth;
    }

    public int getYSize() {
        return imageHeight;
    }

    protected void containerTick() {}

    @Override
    public void tick() {
        containerTick();
    }
}
