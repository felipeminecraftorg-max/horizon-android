// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.inventory;

/** A menu: its slots and its kind. Horizon moves the items; lane 3 keeps the layout. */
public abstract class AbstractContainerMenu {
    public final net.minecraft.core.NonNullList<Slot> slots = net.minecraft.core.NonNullList.create();
    public final int containerId;
    private final MenuType<?> menuType;

    protected AbstractContainerMenu(MenuType<?> menuType, int containerId) {
        this.menuType = menuType;
        this.containerId = containerId;
    }

    public MenuType<?> getType() {
        return menuType;
    }

    protected Slot addSlot(Slot slot) {
        slot.index = slots.size();
        slots.add(slot);
        return slot;
    }

    public Slot getSlot(int index) {
        return slots.get(index);
    }

    public abstract boolean stillValid(net.minecraft.world.entity.player.Player player);

    public net.minecraft.world.item.ItemStack quickMoveStack(net.minecraft.world.entity.player.Player player, int index) {
        return net.minecraft.world.item.ItemStack.EMPTY;
    }

    public void removed(net.minecraft.world.entity.player.Player player) {}

    public void broadcastChanges() {}

    public void slotsChanged(net.minecraft.world.Container container) {}

    protected static boolean stillValid(ContainerLevelAccess access, net.minecraft.world.entity.player.Player player,
            net.minecraft.world.level.block.Block block) {
        return true;
    }

    protected boolean moveItemStackTo(net.minecraft.world.item.ItemStack stack, int start, int end, boolean reverse) {
        return false;
    }
}
