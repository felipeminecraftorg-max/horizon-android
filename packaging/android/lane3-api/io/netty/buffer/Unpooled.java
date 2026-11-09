// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package io.netty.buffer;

/** New buffers. */
public final class Unpooled {
    private Unpooled() {}

    public static ByteBuf buffer() {
        return new ByteBuf();
    }

    public static ByteBuf buffer(int capacity) {
        return new ByteBuf();
    }
}
