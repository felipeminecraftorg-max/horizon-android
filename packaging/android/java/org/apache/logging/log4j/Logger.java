// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package org.apache.logging.log4j;

/** The part of Log4j's Logger mods call. */
public interface Logger {
    String getName();

    void debug(String message);

    void debug(String message, Object... params);

    void info(String message);

    void info(String message, Object... params);

    void warn(String message);

    void warn(String message, Object... params);

    void error(String message);

    void error(String message, Object... params);

    void error(String message, Throwable t);

    void fatal(String message);
}
