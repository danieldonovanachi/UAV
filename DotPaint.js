<script>
        // ============================================
        // DOT PAINT LOGIC (Calligraphy-based drawing)
        // ============================================
        
        var dotDrawingLayer;
        var dotBrushAngle = 0;
        var dotPrevAngle = 0;
        var dotBrushColors = [];
        var dotNumSections = 5; // Standard Anzahl der Pinselstriche
        var dotFollowerX, dotFollowerY;
        var dotPrevFollowerX, dotPrevFollowerY;
        var dotRotationOffset; 
        var dotPrevBrushAngle;
        var dotIsFirstStroke = true;
        var dotTotalLength = 200; // Standard Größe
        var dotFollowerActive = false;
        var dotBgColor;
        var dotParentContainer;
        var dotMainCanvas;
        var dotP5Instance = null;
        var dotP5Canvas = null;

        // Farbpalette (Identisch zu Grid Paint)
        var dotColorPalette = [
          [255, 255, 255], // Index 0: Weiß (Radierer)
          [0, 0, 0],       // Index 1: Schwarz
          [255, 0, 0],     // Index 2: Rot
          [0, 255, 0],     // Index 3: Grün
          [0, 0, 255],     // Index 4: Blau
          [255, 255, 0],   // Index 5: Gelb
          [255, 0, 255]    // Index 6: Magenta
        ];
        var dotCurrentColorIndex = 1; // Standardmäßig Schwarz starten (nicht Radierer)

        // Helper: Generiert das Array der aktuellen Pinsel-Farben basierend auf der Auswahl
        function dotGenerateBrushColors(count) {
          let colors = [];
          let selectedColor = dotColorPalette[dotCurrentColorIndex];
          for (let i = 0; i < count; i++) {
            colors.push(selectedColor);
          }
          return colors;
        }
        
        // Dot Paint p5.js Sketch
        var dotP5Sketch = function(p) {
          
          p.setup = function() {
            dotParentContainer = document.getElementById('dotCanvasContainer');
            if (!dotParentContainer) return;
            
            // Canvas erstellen
            dotMainCanvas = p.createCanvas(dotParentContainer.clientWidth, dotParentContainer.clientHeight);
            p.pixelDensity(2); // Für scharfe Linien auf Retina/iPad

            dotBgColor = p.color(255, 255, 255); // Hintergrund Weiß

            // PGraphics Layer für die permanente Zeichnung
            dotDrawingLayer = p.createGraphics(p.width, p.height);
            dotDrawingLayer.pixelDensity(2);
            dotDrawingLayer.background(255, 0); // Transparent oder Weiß initialisieren
            // dotDrawingLayer.clear(); // Wichtig für Transparenz

            p.background(dotBgColor);
            
            // Rotation Offset (90 Grad)
            dotRotationOffset = p.HALF_PI;
            
            // Initiale Farben setzen
            dotBrushColors = dotGenerateBrushColors(dotNumSections);
            
            // Follower initialisieren (Mitte)
            dotFollowerX = p.width / 2;
            dotFollowerY = p.height / 2;
            dotPrevFollowerX = dotFollowerX;
            dotPrevFollowerY = dotFollowerY;
            
            // UI Setup
            dotSetupColorPalette();
            // Initiale Farbe im UI setzen
            dotSetColor(dotCurrentColorIndex);
            
            // Touch Action deaktivieren für besseres Zeichnen
            dotMainCanvas.elt.style.touchAction = 'none';
            dotMainCanvas.elt.style.pointerEvents = 'auto';
          };
          
          p.draw = function() {
            // Resize Check
            if (dotParentContainer && p.width != dotParentContainer.clientWidth) {
              p.windowResized();
            }
            
            // Hauptcanvas löschen und DrawingLayer anzeigen
            p.background(dotBgColor);
            p.image(dotDrawingLayer, 0, 0, p.width, p.height);
            
            // Maus/Touch Position holen
            let targetX = p.mouseX;
            let targetY = p.mouseY;

            // Touch Support prüfen
            if (p.touches.length > 0) {
                targetX = p.touches[0].x;
                targetY = p.touches[0].y;
                dotFollowerActive = true;
            } else if (p.mouseIsPressed) {
                dotFollowerActive = true;
            } else {
                // Wenn nicht gedrückt, Follower deaktivieren um "Nachziehen" zu stoppen
                dotFollowerActive = false;
                dotIsFirstStroke = true; // Reset für nächsten Strich
            }
            
            // Follower Logik (Lerp für geschmeidige Bewegung)
            if (dotFollowerActive) {
               // Lerp Faktor: 0.1 = sehr weich/langsam, 0.5 = direkt
               let lerpFactor = 0.3; 
               dotFollowerX = p.lerp(dotFollowerX, targetX, lerpFactor);
               dotFollowerY = p.lerp(dotFollowerY, targetY, lerpFactor);
            } else {
               // Wenn inaktiv, Position direkt setzen damit es beim nächsten Klick nicht springt
               dotFollowerX = targetX;
               dotFollowerY = targetY;
            }
            
            // Winkelberechnung
            let sectionLength = dotTotalLength / dotNumSections;
            
            let dx = dotFollowerX - dotPrevFollowerX;
            let dy = dotFollowerY - dotPrevFollowerY;
            let distance = p.dist(dotPrevFollowerX, dotPrevFollowerY, dotFollowerX, dotFollowerY);
            
            let currentAngle = p.atan2(dy, dx);
            
            // Pinsel-Rotation basierend auf Bewegung
            if (distance > 0.5) { // Nur rotieren wenn Bewegung da ist
              let targetAngle = currentAngle;
              
              // Kürzesten Weg für die Rotation finden
              let angleDiff1 = targetAngle - dotBrushAngle;
              let angleDiff2 = (targetAngle + p.PI) - dotBrushAngle;
              
              angleDiff1 = p.atan2(p.sin(angleDiff1), p.cos(angleDiff1));
              angleDiff2 = p.atan2(p.sin(angleDiff2), p.cos(angleDiff2));
              
              let angleDiff = p.abs(angleDiff1) < p.abs(angleDiff2) ? angleDiff1 : angleDiff2;
              
              // Glättung der Rotation
              dotBrushAngle += angleDiff * 0.1;
            }
            
            dotPrevAngle = currentAngle;
            
            // ZEICHNEN
            // Nur zeichnen, wenn aktiv, Bewegung stattfindet und nicht der allererste Frame des Klicks
            if (dotFollowerActive && !dotIsFirstStroke && distance > 0.5) {
              let halfLength = dotTotalLength / 2;
              
              // Für jede Sektion des Pinsels
              for (let i = 0; i < dotNumSections; i++) {
                let segStart = -halfLength + i * sectionLength;
                let segEnd = -halfLength + (i + 1) * sectionLength;
                
                // Zeichne auf den permanenten Layer
                dotDrawBrushSegment(dotDrawingLayer, 
                                dotPrevFollowerX, dotPrevFollowerY, dotPrevBrushAngle,
                                dotFollowerX, dotFollowerY, dotBrushAngle,
                                segStart, segEnd, dotBrushColors[i], p);
              }
            }
            
            // Werte für nächsten Frame speichern
            if (dotFollowerActive) {
                dotPrevFollowerX = dotFollowerX;
                dotPrevFollowerY = dotFollowerY;
                dotPrevBrushAngle = dotBrushAngle;
                dotIsFirstStroke = false;
            } else {
                // Reset Positionen wenn nicht gezeichnet wird
                dotPrevFollowerX = dotFollowerX;
                dotPrevFollowerY = dotFollowerY;
            }
          };
          
          p.windowResized = function() {
            if (!dotParentContainer) return;
            p.resizeCanvas(dotParentContainer.clientWidth, dotParentContainer.clientHeight);
            
            // Layer retten beim Resize
            let temp = p.createGraphics(p.width, p.height);
            temp.image(dotDrawingLayer, 0, 0);
            dotDrawingLayer = p.createGraphics(p.width, p.height);
            dotDrawingLayer.pixelDensity(2);
            dotDrawingLayer.image(temp, 0, 0);
            temp.remove();
          };
          
          // Globale Referenz speichern
          window.dotP5Instance = p;
          dotP5Canvas = dotMainCanvas;
        };
        
        // Die Zeichenfunktion (1:1 Logik + Radierer Anpassung)
        function dotDrawBrushSegment(target, x1, y1, angle1, x2, y2, angle2, segmentStart, segmentEnd, segmentColor, p) {
          let finalAngle1 = angle1 + dotRotationOffset;
          let finalAngle2 = angle2 + dotRotationOffset;
          
          // --- RADIERER LOGIK START ---
          // Index 0 ist Weiß/Radierer
          if (dotCurrentColorIndex === 0) {
            target.erase(); // p5.js erase Modus aktivieren
          } else {
            target.noErase();
          }
          // --- RADIERER LOGIK ENDE ---
          
          // Die 4 Ecken des Parallelogramms berechnen
          let p1x = x1 + p.cos(finalAngle1) * segmentStart;
          let p1y = y1 + p.sin(finalAngle1) * segmentStart;
          let p2x = x1 + p.cos(finalAngle1) * segmentEnd;
          let p2y = y1 + p.sin(finalAngle1) * segmentEnd;
          
          let p3x = x2 + p.cos(finalAngle2) * segmentEnd;
          let p3y = y2 + p.sin(finalAngle2) * segmentEnd;
          let p4x = x2 + p.cos(finalAngle2) * segmentStart;
          let p4y = y2 + p.sin(finalAngle2) * segmentStart;
          
          target.noStroke();
          target.fill(segmentColor[0], segmentColor[1], segmentColor[2]);
          
          // Wenn wir radieren, brauchen wir vielleicht etwas mehr 'Kraft' oder Stroke, 
          // aber fill reicht meistens bei erase()
          target.quad(p1x, p1y, p2x, p2y, p3x, p3y, p4x, p4y);
          
          // Erase Modus sofort wieder beenden
          if (dotCurrentColorIndex === 0) {
            target.noErase();
          }
        }
        
        // UI Controls Functions
        function dotSetColor(index) {
          if (!dotP5Instance) return;
          
          dotCurrentColorIndex = index;
          dotBrushColors = dotGenerateBrushColors(dotNumSections);

          // UI Update
          var palette = document.getElementById('dotColorPalette');
          if (palette) {
            var children = palette.children;
            for (var i = 0; i < children.length; i++) {
              if (i === index) {
                children[i].classList.add('active');
                children[i].style.border = '3px solid #000';
              } else {
                children[i].classList.remove('active');
                children[i].style.border = '2px solid rgba(0,0,0,0.3)';
              }
            }
          }
        }
        
        function dotSetSize(val) {
          if (!dotP5Instance) return;
          var p = dotP5Instance;
          var minSize = 50;
          var maxSize = 600;

          if (val == "+") {
            dotTotalLength += 50;
          } else if (val == "-") {
            dotTotalLength -= 50;
          }
          dotTotalLength = p.constrain(dotTotalLength, minSize, maxSize);
          
          // Optional: Anzahl der Sektionen anpassen für Dichte
          // dotNumSections = p.floor(p.map(dotTotalLength, minSize, maxSize, 3, 10));
          // dotBrushColors = dotGenerateBrushColors(dotNumSections);
        }
        
        function dotReset() {
          if (!dotP5Instance || !dotDrawingLayer) return;
          
          dotDrawingLayer.clear(); // Layer leeren
          // Oder mit Weiß füllen:
          // dotDrawingLayer.background(255);
          
          // Farbe auf Schwarz zurücksetzen (optional)
          dotSetColor(1);
        }
        
        function dotSaveIMG() {
          if (!dotP5Instance) return;
          
          var now = new Date();
          // Einfacher Timestamp
          var filename = "DOT_" + now.getTime() + ".png";
          
          dotP5Instance.save(dotMainCanvas, filename);
        }
        
        function dotSetupColorPalette() {
          var palette = document.getElementById('dotColorPalette');
          if (!palette) return;
          palette.innerHTML = '';
          
          for (var i = 0; i < dotColorPalette.length; i++) {
            var colorDiv = document.createElement('div');
            colorDiv.className = 'grid-color-icon'; // Klasse wiederverwenden für Styling
            // Initial Active State
            if (i === dotCurrentColorIndex) {
               colorDiv.classList.add('active');
               colorDiv.style.border = '3px solid #000';
            }
            
            colorDiv.style.width = '35px';
            colorDiv.style.height = '35px';
            colorDiv.style.backgroundColor = 'rgb(' + dotColorPalette[i][0] + ',' + dotColorPalette[i][1] + ',' + dotColorPalette[i][2] + ')';
            
            // Click Handler
            colorDiv.onclick = (function(index) {
              return function(e) {
                e.stopPropagation();
                dotSetColor(index);
              };
            })(i);
            
            // Touch Handler
            colorDiv.ontouchstart = (function(index) {
              return function(e) {
                e.stopPropagation();
                dotSetColor(index);
              };
            })(i);
            
            palette.appendChild(colorDiv);
          }
        }
    </script>