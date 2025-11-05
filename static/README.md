# Static Files Directory

This directory contains static assets served by the VectDB web server.

## Files

- **index.html** - Main web interface HTML
- **build-info.js** - Auto-generated build information (created by `scripts/build.sh`)
- **favicon.ico** (optional) - Browser favicon

## Adding a Favicon

To add a custom favicon to the web interface:

1. Create or obtain a `favicon.ico` file (16x16 or 32x32 pixels)
2. Place it in this directory: `static/favicon.ico`
3. Rebuild and restart the server:
   ```bash
   ./scripts/build.sh
   ./scripts/serve.sh
   ```

The favicon will be automatically served at `/favicon.ico`.

## Build Info

The `build-info.js` file is automatically generated during build and contains:
- Build hostname
- Git commit SHA
- Build timestamp (ISO 8601 format)

This information is displayed in the web UI footer.

**Note**: `build-info.js` is excluded from git tracking (see `.gitignore`) as it's regenerated on each build.
