# API Reference

<div id="swagger-ui"></div>

<link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui.css" />
<style>
  /* Force light theme for Swagger UI regardless of page theme */
  #swagger-ui {
    background: #fafafa;
    padding: 20px;
    border-radius: 4px;
  }

  /* Ensure all text is readable */
  #swagger-ui .swagger-ui,
  #swagger-ui .info,
  #swagger-ui .scheme-container,
  #swagger-ui .opblock-tag,
  #swagger-ui .opblock,
  #swagger-ui .responses-inner,
  #swagger-ui .parameters,
  #swagger-ui .model-container {
    color: #3b4151 !important;
    background: #fafafa !important;
  }

  /* Fix description text */
  #swagger-ui .info .description,
  #swagger-ui .opblock-description-wrapper p,
  #swagger-ui .opblock-summary-description {
    color: #3b4151 !important;
  }

  /* Fix headers */
  #swagger-ui .opblock .opblock-summary-operation-id,
  #swagger-ui .opblock .opblock-summary-path,
  #swagger-ui .opblock-tag {
    color: #3b4151 !important;
  }
</style>
<script src="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui-bundle.js"></script>
<script src="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui-standalone-preset.js"></script>
<script>
(function() {
  function renderSwagger() {
    const container = document.getElementById('swagger-ui');
    if (container && typeof SwaggerUIBundle !== 'undefined') {
      const ui = SwaggerUIBundle({
        url: '../../openapi.yml',
        dom_id: '#swagger-ui',
        deepLinking: true,
        presets: [
          SwaggerUIBundle.presets.apis,
          SwaggerUIStandalonePreset
        ],
        layout: "BaseLayout"
      });
      window.ui = ui;
    }
  }

  // Handle both initial load and MkDocs navigation
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', renderSwagger);
  } else {
    renderSwagger();
  }

  // Re-render on MkDocs instant navigation
  document.addEventListener('DOMContentLoaded', function() {
    document$.subscribe(renderSwagger);
  });
})();
</script>
