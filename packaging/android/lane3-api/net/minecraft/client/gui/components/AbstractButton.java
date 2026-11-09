// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui.components;

/** A widget a click presses. */
public abstract class AbstractButton extends AbstractWidget {
    public AbstractButton(int x, int y, int width, int height, net.minecraft.network.chat.Component message) {
        super(x, y, width, height, message);
    }

    public abstract void onPress();

    @Override
    public void onClick(double mouseX, double mouseY) {
        onPress();
    }

    @Override
    protected void renderWidget(net.minecraft.client.gui.GuiGraphics graphics, int mouseX, int mouseY, float partialTick) {
        graphics.horizonButton(getX(), getY(), width, height, isHoveredOrFocused(), active,
                getMessage() == null ? "" : getMessage().getString());
    }
}
