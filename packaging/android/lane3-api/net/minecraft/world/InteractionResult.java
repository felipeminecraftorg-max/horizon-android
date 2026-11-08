// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world;

/** 1.21.1's InteractionResult (an enum until 1.21.2). */
public enum InteractionResult {
    SUCCESS,
    SUCCESS_NO_ITEM_USED,
    CONSUME,
    CONSUME_PARTIAL,
    PASS,
    FAIL;

    public boolean consumesAction() {
        return this == SUCCESS || this == CONSUME || this == CONSUME_PARTIAL || this == SUCCESS_NO_ITEM_USED;
    }

    public boolean shouldSwing() {
        return this == SUCCESS || this == SUCCESS_NO_ITEM_USED;
    }

    public boolean indicateItemUse() {
        return this == SUCCESS || this == CONSUME;
    }

    public static InteractionResult sidedSuccess(boolean clientSide) {
        return clientSide ? SUCCESS : CONSUME;
    }
}
