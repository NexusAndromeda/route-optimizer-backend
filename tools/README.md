# 🛠️ Tools - Scripts de Análisis y Desarrollo

Esta carpeta contiene scripts y herramientas de desarrollo para análisis de datos de Colis Privé.

## 📁 Archivos

### 🔍 **Analizadores JSON**
- **`analyze_json.rs`** - Analizador principal de respuestas JSON de Colis Privé
- **`analyze_json_bin.rs`** - Versión ejecutable standalone del analizador

### 📦 **Clasificadores de Entregas**  
- **`classify_deliveries.rs`** - Clasifica paquetes por tipo de entrega y ubicación
- **`optimize_extraction.rs`** - Optimiza extracción de datos de respuestas API

## 📊 **Estadísticas**
- **4 scripts** de análisis y procesamiento
- **96KB** de herramientas especializadas  
- **Listos para reutilizar** en análisis futuros

## 🚀 Cómo usar

Cuando necesites analizar nuevos JSONs de Colis Privé:

1. Coloca los archivos JSON en una carpeta temporal
2. Ejecuta el script con los parámetros necesarios
3. Revisa los reportes generados

## 📝 Para Cursor AI

Estos scripts fueron creados durante el reverse engineering de la API de Colis Privé. Contienen:

- **Estructuras de datos** completas de respuestas de Colis Privé
- **Parsing inteligente** de campos JSON complejos  
- **Análisis de patrones** en datos de entrega
- **Generación de reportes** de análisis

**Útiles para:** Entender nuevas respuestas de API, debuggear datos, analizar patrones de entregas.