// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.items;

import net.minecraft.world.item.ItemStack;

/** A menu's slot over NeoForge's item slots. */
public class SlotItemHandler extends net.minecraft.world.inventory.Slot {
    private final IItemHandler itemHandler;
    private final int index;

    public SlotItemHandler(IItemHandler itemHandler, int index, int x, int y) {
        super(null, index, x, y);
        this.itemHandler = itemHandler;
        this.index = index;
    }

    public IItemHandler getItemHandler() {
        return itemHandler;
    }

    @Override
    public ItemStack getItem() {
        return itemHandler == null ? ItemStack.EMPTY : itemHandler.getStackInSlot(index);
    }

    @Override
    public void set(ItemStack stack) {
        if (itemHandler instanceof IItemHandlerModifiable m) m.setStackInSlot(index, stack);
    }

    @Override
    public boolean mayPlace(ItemStack stack) {
        return itemHandler != null && itemHandler.isItemValid(index, stack);
    }
}
