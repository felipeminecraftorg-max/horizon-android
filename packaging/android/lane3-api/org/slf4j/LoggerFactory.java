// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package org.slf4j;

public final class LoggerFactory {
    private LoggerFactory() {}

    public static Logger getLogger(String name) {
        return new dev.horizon.android.lane3.Log(name);
    }

    public static Logger getLogger(Class<?> type) {
        return new dev.horizon.android.lane3.Log(type.getSimpleName());
    }
}
