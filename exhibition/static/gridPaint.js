// Main
var bgColor;

// Specific
var imgDrawing;
var gridSizePreset = 75;
var gridSizeSteps = 25;
var gridSize;
var brushColor;
var gridElementColor;
var rasterPoints = [];
var magicNr = 0.553;
var gridXElements;
var gridYElements;

// Extended color palette - more colors than original
var colorPalette = [
  [0, 0, 0],       // Black
  [255, 255, 255], // White
  [255, 0, 0],     // Red
  [0, 255, 0],     // Green
  [0, 0, 255],     // Blue
  [255, 255, 0],   // Yellow
  [255, 0, 255],   // Magenta
  [0, 255, 255],   // Cyan
  [255, 136, 0],   // Orange
  [136, 0, 255],   // Purple
  [0, 255, 136],   // Mint
  [255, 0, 136],   // Pink
  [136, 136, 136], // Gray
  [255, 68, 68],   // Light Red
  [68, 255, 68],   // Light Green
  [68, 68, 255],   // Light Blue
  [255, 255, 68],  // Light Yellow
  [255, 68, 255],  // Light Magenta
  [68, 255, 255],  // Light Cyan
  [255, 170, 0],   // Light Orange
  [170, 0, 255],   // Light Purple
  [0, 255, 170],   // Light Mint
  [255, 0, 170],   // Light Pink
  [170, 170, 170], // Light Gray
  [255, 102, 102], // Lighter Red
  [102, 255, 102], // Lighter Green
  [102, 102, 255], // Lighter Blue
  [255, 255, 102], // Lighter Yellow
  [255, 102, 255], // Lighter Magenta
  [102, 255, 255], // Lighter Cyan
  [204, 0, 0],     // Dark Red
  [0, 204, 0],     // Dark Green
  [0, 0, 204],     // Dark Blue
  [204, 204, 0],   // Dark Yellow
  [204, 0, 204],   // Dark Magenta
  [0, 204, 204],   // Dark Cyan
  [153, 0, 0],     // Darker Red
  [0, 153, 0],     // Darker Green
  [0, 0, 153],     // Darker Blue
  [153, 153, 0]    // Darker Yellow
];
var currentColorIndex = 0;

//--------------------------------------------------

function preload() {
  bgColor = color(255);
  interfaceColor = color(0);
  brushColor = color(255);
  gridElementColor = color(0);
}

function setup() {
  var cnv = createCanvas(windowWidth, windowHeight);
  pixelDensity(2);
  document.body.style.backgroundColor = bgColor;
  
  // Ensure canvas receives touch events
  cnv.elt.style.touchAction = 'none';
  cnv.elt.style.pointerEvents = 'auto';
  cnv.elt.style.position = 'absolute';
  cnv.elt.style.top = '0';
  cnv.elt.style.left = '0';
  cnv.elt.style.zIndex = '1';
  
  // Prevent default touch behaviors
  cnv.elt.addEventListener('touchstart', function(e) {
    e.preventDefault();
  }, { passive: false });
  cnv.elt.addEventListener('touchmove', function(e) {
    e.preventDefault();
  }, { passive: false });
  cnv.elt.addEventListener('touchend', function(e) {
    e.preventDefault();
  }, { passive: false });
  
  if (isTouchDevice() == true) {
    gridSizePreset *= 0.5;
    gridSizeSteps *= 0.5;
  }
  gridSize = gridSizePreset;
  reset();
  imageMode(CENTER);
  setupColorPalette();
}

function setupColorPalette() {
  var palette = document.getElementById('colorPalette');
  palette.innerHTML = '';
  
  for (var i = 0; i < colorPalette.length; i++) {
    var colorDiv = document.createElement('div');
    colorDiv.className = 'color-icon';
    if (i === currentColorIndex) {
      colorDiv.classList.add('active');
    }
    colorDiv.style.width = '35px';
    colorDiv.style.height = '35px';
    colorDiv.style.backgroundColor = 'rgb(' + colorPalette[i][0] + ',' + colorPalette[i][1] + ',' + colorPalette[i][2] + ')';
    colorDiv.onclick = (function(index) {
      return function(e) {
        e.stopPropagation();
        setColor(index);
      };
    })(i);
    colorDiv.ontouchstart = (function(index) {
      return function(e) {
        e.stopPropagation();
        setColor(index);
      };
    })(i);
    palette.appendChild(colorDiv);
  }
}

function draw() {
  display();
  //print(frameRate());
}

//--------------------------------------------------

function display() {
  image(imgDrawing, width / 2, height / 2);
  updateRasterPoints();
  background(bgColor);
  displayRasterPoints();
}

//--------------------------------------------------
var rot = 0.0;
var isDrawing = false;
var lastMouseX = 0;
var lastMouseY = 0;

function mousePressed() {
  // Check if clicking on UI elements
  if (mouseX > width - 205 && mouseY < 70) {
    return;
  }
  // Check if clicking on color palette area (bottom center)
  var palette = document.getElementById('colorPalette');
  if (palette) {
    var rect = palette.getBoundingClientRect();
    var canvasRect = document.getElementById('defaultCanvas0');
    if (canvasRect) {
      var canvasBounds = canvasRect.getBoundingClientRect();
      var paletteX = rect.left - canvasBounds.left;
      var paletteY = rect.top - canvasBounds.top;
      if (mouseX >= paletteX && mouseX <= paletteX + rect.width && mouseY >= paletteY && mouseY <= paletteY + rect.height) {
        return;
      }
    }
  }
  // Check if clicking on grid size buttons (bottom right)
  if (mouseX > width - 70 && mouseY > height - 260) {
    return;
  }
  
  isDrawing = true;
  lastMouseX = mouseX;
  lastMouseY = mouseY;
  drawImg(mouseX, mouseY, mouseX, mouseY);
}

function mouseDragged() {
  if (!isDrawing) return;
  
  // Check if dragging over UI elements
  if (mouseX > width - 205 && mouseY < 70) {
    return;
  }
  if (mouseX > width - 70 && mouseY > height - 260) {
    return;
  }
  
  drawImg(mouseX, mouseY, lastMouseX, lastMouseY);
  lastMouseX = mouseX;
  lastMouseY = mouseY;
}

function mouseReleased() {
  isDrawing = false;
}

function touchStarted() {
  // Convert touch to mouse coordinates
  if (touches.length > 0) {
    var touch = touches[0];
    mouseX = touch.x;
    mouseY = touch.y;
    pmouseX = touch.x;
    pmouseY = touch.y;
    
    // Check if touching UI elements
    if (mouseX > width - 205 && mouseY < 70) {
      return false;
    }
    var palette = document.getElementById('colorPalette');
    if (palette) {
      var rect = palette.getBoundingClientRect();
      var canvasRect = document.getElementById('defaultCanvas0');
      if (canvasRect) {
        var canvasBounds = canvasRect.getBoundingClientRect();
        var paletteX = rect.left - canvasBounds.left;
        var paletteY = rect.top - canvasBounds.top;
        if (mouseX >= paletteX && mouseX <= paletteX + rect.width && mouseY >= paletteY && mouseY <= paletteY + rect.height) {
          return false;
        }
      }
    }
    if (mouseX > width - 70 && mouseY > height - 260) {
      return false;
    }
    
    isDrawing = true;
    lastMouseX = mouseX;
    lastMouseY = mouseY;
    drawImg(mouseX, mouseY, mouseX, mouseY);
  }
  return false; // Prevent default
}

function touchMoved() {
  if (!isDrawing) return false;
  
  if (touches.length > 0) {
    var touch = touches[0];
    mouseX = touch.x;
    mouseY = touch.y;
    
    // Check if dragging over UI elements
    if (mouseX > width - 205 && mouseY < 70) {
      return false;
    }
    if (mouseX > width - 70 && mouseY > height - 260) {
      return false;
    }
    
    drawImg(mouseX, mouseY, lastMouseX, lastMouseY);
    lastMouseX = mouseX;
    lastMouseY = mouseY;
  }
  return false; // Prevent default
}

function touchEnded() {
  isDrawing = false;
  return false; // Prevent default
}

function drawImg(x, y, px, py) {
  if (imgDrawing == null) return;
  imgDrawing.stroke(brushColor);
  imgDrawing.strokeWeight(gridSize);
  imgDrawing.noFill();
  // If px/py are same as x/y (first click), draw a point
  if (px == x && py == y) {
    imgDrawing.point(x, y);
  } else {
    imgDrawing.line(x, y, px, py);
  }
}

//--------------------------------------------------

function setColor(index) {
  currentColorIndex = index;
  var c = colorPalette[index];
  brushColor = color(c[0], c[1], c[2]);
  gridElementColor = color(c[0], c[1], c[2]);
  
  // Update color palette UI
  var palette = document.getElementById('colorPalette');
  var children = palette.children;
  for (var i = 0; i < children.length; i++) {
    if (i === index) {
      children[i].classList.add('active');
    } else {
      children[i].classList.remove('active');
    }
  }
}

function setGridSize(val) {
  // Extended range: 5 to 200 (instead of 50-150)
  var gridSizeMin = 5;   // Smaller than original
  var gridSizeMax = 200; // Bigger than original

  if (val == "+") {
    gridSize += gridSizeSteps;
  } else if (val == "-") {
    gridSize -= gridSizeSteps;
  } else {
    gridSize *= val;
    gridSize = round(gridSize / gridSizeSteps) * gridSizeSteps;
  }
  gridSize = constrain(gridSize, gridSizeMin, gridSizeMax);

  if (gridSize == gridSizeMin) {
    document.getElementById("gridMinus").style.opacity = "0.3";
  } else if (gridSize == gridSizeMax) {
    document.getElementById("gridPlus").style.opacity = "0.3";
  } else {
    document.getElementById("gridMinus").style.opacity = "1.0";
    document.getElementById("gridPlus").style.opacity = "1.0";
  }

  createRasterPoints();
}


//--------------------------------------------------

function gridify(IN) {
  var OUT = int(round(float(IN) / gridSize) * gridSize);
  return OUT;
}

function createRasterPoints() {
  rasterPoints = [];
  gridXElements = floor(width / gridSize) + 2;
  gridYElements = floor(height / gridSize) + 2;
  for (var x = 0; x < gridXElements; x++) {
    rasterPoints[x] = [];
    for (var y = 0; y < gridYElements; y++) {
      var xPos = x * gridSize + ((width - (gridSize * gridXElements)) / 2) + (gridSize / 2);
      var yPos = y * gridSize + ((height - (gridSize * gridYElements)) / 2) + (gridSize / 2);
      rasterPoints[x][y] = new RasterPoint(xPos, yPos);
    }
  }
}

function updateRasterPoints() {
  var tempScreen = get(0, 0, width, height);
  tempScreen.resize(int(width / gridSize) * 4, (height / gridSize) * 4);
  var factor = width / tempScreen.width;

  for (var x = 0; x < gridXElements; x++) {
    for (var y = 0; y < gridYElements; y++) {
      rasterPoints[x][y].update(tempScreen, factor);
    }
  }
}

function displayRasterPoints() {
  for (var x = 0; x < gridXElements; x++) {
    for (var y = 0; y < gridYElements; y++) {
      rasterPoints[x][y].display();
    }
  }
}

//--------------------------------------------------

function reset() {
  imgDrawing = null;
  resizeImages();

  currentColorIndex = 0;
  var c = colorPalette[0];
  brushColor = color(c[0], c[1], c[2]);
  gridElementColor = color(c[0], c[1], c[2]);
  
  // Update color palette UI
  var palette = document.getElementById('colorPalette');
  if (palette) {
    var children = palette.children;
    for (var i = 0; i < children.length; i++) {
      if (i === 0) {
        children[i].classList.add('active');
      } else {
        children[i].classList.remove('active');
      }
    }
  }

  if (imgDrawing != null) {
    imgDrawing.background(0);
  }
}

function resizeImages() {
  if (imgDrawing != null) {
    var imgDrawingTemp = createGraphics(windowWidth, windowHeight);
    var factor;
    if (windowWidth > imgDrawing.width || windowHeight > imgDrawing.height) factor = max([(windowWidth / imgDrawing.width), (windowHeight / imgDrawing.height)]);
    else factor = min([(windowWidth / imgDrawing.width), (windowHeight / imgDrawing.height)]);
    imgDrawingTemp.background(0);
    imgDrawingTemp.imageMode(CENTER);
    imgDrawingTemp.image(imgDrawing, imgDrawingTemp.width / 2, imgDrawingTemp.height / 2, imgDrawing.width * factor, imgDrawing.height * factor);
    imgDrawingTemp.imageMode(CORNER);
    imgDrawing.remove();
    imgDrawing = null;
    imgDrawing = imgDrawingTemp;
    imgDrawingTemp.remove();
    imgDrawingTemp = null;
    setGridSize(factor);
  } else {
    imgDrawing = createGraphics(width, height);
  }
  createRasterPoints();
}

//--------------------------------------------------

function saveIMG() {
  let filename = "IMG_" + year() + '-' + month() + '-' + day() + '_' + hour() + '-' + minute() + '-' + second() + '_' + round(millis()) + ".png";
  display();
  save("" + filename);
  
  // Try to send to server if in iframe
  try {
    if (window.parent && window.parent.ws && window.parent.ws.readyState === WebSocket.OPEN) {
      var canvasData = document.getElementById('defaultCanvas0').toDataURL('image/png');
      window.parent.ws.send(JSON.stringify({
        type: 'image_submit',
        image: canvasData,
        style: 'grid'
      }));
    }
  } catch (e) {
    // Not in iframe or no WebSocket, ignore
  }
}
