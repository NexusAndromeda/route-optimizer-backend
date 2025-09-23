-- =====================================================
-- DATOS DE EJEMPLO PARA APP_VERSIONS
-- =====================================================
-- Este archivo contiene datos de ejemplo para probar el sistema de actualizaciones

-- Insertar versión actual (Alpha)
INSERT INTO app_versions (
    version_name,
    version_code,
    build_type,
    apk_path,
    apk_size_bytes,
    apk_checksum,
    download_url,
    release_notes,
    release_notes_fr,
    release_notes_en,
    force_update,
    min_supported_version,
    min_android_version,
    max_android_version,
    supported_architectures,
    is_active,
    is_beta,
    published_at
) VALUES (
    '1.0.0-alpha',
    1,
    'alpha',
    '/var/www/delivery-routing/releases/app-v1.0.0-alpha.apk',
    52428800, -- 50MB ejemplo
    'a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6',
    'http://localhost:3000/api/mobile/download/apk',
    'Versión Alpha inicial con funcionalidades básicas de mapa y paquetes',
    'Version Alpha initiale avec fonctionnalités de base de carte et colis',
    'Initial Alpha version with basic map and package functionality',
    FALSE,
    '1.0.0-alpha',
    21, -- Android 5.0
    34, -- Android 14
    ARRAY['arm64-v8a', 'armeabi-v7a'],
    TRUE,
    TRUE,
    NOW()
);

-- Insertar versión de ejemplo más reciente
INSERT INTO app_versions (
    version_name,
    version_code,
    build_type,
    apk_path,
    apk_size_bytes,
    apk_checksum,
    download_url,
    release_notes,
    release_notes_fr,
    release_notes_en,
    force_update,
    min_supported_version,
    min_android_version,
    max_android_version,
    supported_architectures,
    is_active,
    is_beta,
    published_at
) VALUES (
    '1.0.1',
    2,
    'release',
    '/var/www/delivery-routing/releases/app-v1.0.1.apk',
    53687091, -- 51MB ejemplo
    'b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6a7',
    'http://localhost:3000/api/mobile/download/apk',
    'Nueva versión con mejoras en el mapa, sistema de actualización automática y corrección de bugs',
    'Nouvelle version avec améliorations de la carte, système de mise à jour automatique et correction de bugs',
    'New version with map improvements, automatic update system and bug fixes',
    FALSE,
    '1.0.0-alpha',
    21, -- Android 5.0
    34, -- Android 14
    ARRAY['arm64-v8a', 'armeabi-v7a'],
    TRUE,
    FALSE,
    NOW()
);

-- Insertar versión de ejemplo con actualización forzada
INSERT INTO app_versions (
    version_name,
    version_code,
    build_type,
    apk_path,
    apk_size_bytes,
    apk_checksum,
    download_url,
    release_notes,
    release_notes_fr,
    release_notes_en,
    force_update,
    min_supported_version,
    min_android_version,
    max_android_version,
    supported_architectures,
    is_active,
    is_beta,
    published_at
) VALUES (
    '1.0.2',
    3,
    'release',
    '/var/www/delivery-routing/releases/app-v1.0.2.apk',
    54525952, -- 52MB ejemplo
    'c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6a7b8',
    'http://localhost:3000/api/mobile/download/apk',
    'ACTUALIZACIÓN CRÍTICA: Corrección de vulnerabilidades de seguridad y mejoras de rendimiento',
    'MISE À JOUR CRITIQUE: Correction des vulnérabilités de sécurité et améliorations de performance',
    'CRITICAL UPDATE: Security vulnerability fixes and performance improvements',
    TRUE, -- ACTUALIZACIÓN FORZADA
    '1.0.0-alpha',
    21, -- Android 5.0
    34, -- Android 14
    ARRAY['arm64-v8a', 'armeabi-v7a'],
    TRUE,
    FALSE,
    NOW()
);

-- Insertar versión beta para testing
INSERT INTO app_versions (
    version_name,
    version_code,
    build_type,
    apk_path,
    apk_size_bytes,
    apk_checksum,
    download_url,
    release_notes,
    release_notes_fr,
    release_notes_en,
    force_update,
    min_supported_version,
    min_android_version,
    max_android_version,
    supported_architectures,
    is_active,
    is_beta,
    published_at
) VALUES (
    '1.1.0-beta',
    4,
    'beta',
    '/var/www/delivery-routing/releases/app-v1.1.0-beta.apk',
    55574528, -- 53MB ejemplo
    'd4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6a7b8c9',
    'http://localhost:3000/api/mobile/download/apk',
    'Versión Beta con nuevas funcionalidades: optimización de rutas, notificaciones push y modo offline',
    'Version Beta avec nouvelles fonctionnalités: optimisation des itinéraires, notifications push et mode hors ligne',
    'Beta version with new features: route optimization, push notifications and offline mode',
    FALSE,
    '1.0.1',
    21, -- Android 5.0
    34, -- Android 14
    ARRAY['arm64-v8a', 'armeabi-v7a'],
    TRUE,
    TRUE,
    NOW()
);

-- Verificar que los datos se insertaron correctamente
SELECT 
    version_name,
    version_code,
    build_type,
    force_update,
    is_active,
    is_beta,
    published_at
FROM app_versions 
ORDER BY version_code DESC;
