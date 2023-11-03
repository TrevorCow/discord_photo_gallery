interface GalleryPage {
    page_title: string;
    galleries: Gallery[];
    page_built_time: string;
}

interface Gallery {
    gallery_title: string;
    gallery_picture_infos: GalleryPicture[];
}

interface GalleryPicture {
    description: string,
    discord_url: URL,
    thumbnail_url: URL,
}

function from_url() {
    const url_params = new URLSearchParams(location.search);
    let test = url_params.get('gallery_page_info');
    const gallery_page_from_url = JSON.parse(test) as GalleryPage;

    build_dom_from_gallery_page(gallery_page_from_url);
}

function build_dom_from_gallery_page(gallery_page: GalleryPage) {
    // Get galleries div
    // For each gallery add h1
    // then add images
    let galleries_div = document.getElementById("galleries") as HTMLDivElement;
    galleries_div.innerHTML = "";

    let galleries_page_title_h1 = document.createElement("h1") as HTMLHeadingElement;
    galleries_page_title_h1.innerText = gallery_page.page_title;
    galleries_div.appendChild(galleries_page_title_h1);

    for (let gallery of gallery_page.galleries) {
        let gallery_title_h1 = document.createElement("h1") as HTMLHeadingElement;
        gallery_title_h1.textContent = gallery.gallery_title;
        galleries_div.appendChild(gallery_title_h1);

        let gallery_div = document.createElement("div") as HTMLDivElement;
        gallery_div.classList.add("gallery")
        for (let gallery_image of gallery.gallery_picture_infos) {
            let img = document.createElement("img");
            if (gallery_image.description) {
                img.dataset.disc = gallery_image.description;
            }
            img.dataset.discord_url = gallery_image.discord_url.toString();
            img.src = gallery_image.thumbnail_url.toString();
            gallery_div.appendChild(img);
        }
        galleries_div.appendChild(gallery_div);
    }

    let gallery_info_h3 = document.getElementById("gallery_info") as HTMLHeadingElement
    gallery_info_h3.innerText = `Gallery was built on ${gallery_page.page_built_time}`;
    on_dom_finished();
}

// Gallery Viewing Code
// document.addEventListener('readystatechange', () => {
//     if (document.readyState === "interactive") {
//         on_dom_finished();
//     }
// });

function on_dom_finished() {
    setup_gallery();
    setup_tool_tips();
}

function show_preview(gimp): void {
    const preview_div = document.querySelector("#preview") as HTMLDivElement;
    if (preview_div.children.length === 0) {
        hide_tooltip_div();
        let previewImg = new Image();
        previewImg.src = gimp.dataset.discord_url;
        preview_div.appendChild(previewImg);
        preview_div.style.display = "block";
    } else {
        preview_div.style.display = "none";
        preview_div.innerHTML = "";
    }

}

function setup_gallery() {
    const allGalleries = document.querySelectorAll(".gallery");
    const allGalleryImages = document.querySelectorAll(".gallery img") as NodeListOf<HTMLImageElement>;

    const resizeAllGalleries = function () {
        allGalleries.forEach(gallery => {
            resizeGalleryItems(gallery)
        });
    }

    const onGalleryImageLoaded = function (gimg) {
        gimg.style.display = "inline";
        gimg.addEventListener("click", function (_event) {
            show_preview(gimg);
        });
        resizeAllGalleries();
    }

    allGalleryImages.forEach(gimg => {
        if (gimg.complete) {
            onGalleryImageLoaded(gimg);
        } else {
            gimg.addEventListener("load", function (_event) {
                onGalleryImageLoaded(gimg);
            });
            gimg.addEventListener('error', function (err) {
                console.log(err);
            });
        }

    });

    window.addEventListener("resize", function (_event) {
        resizeAllGalleries();
    });
}

function resizeGalleryItems(gallery) {
    let children = gallery.querySelectorAll("img");

    children.forEach(child => {
        console.assert(gallery.classList.contains("gallery"))
        let computedGalleryStyle = window.getComputedStyle(gallery);
        let rowHeight = parseInt(computedGalleryStyle.getPropertyValue('grid-auto-rows'));
        let rowGap = parseInt(computedGalleryStyle.getPropertyValue('grid-row-gap'));
        let rowSpan = Math.ceil((child.getBoundingClientRect().height + rowGap) / (rowHeight + rowGap));
        child.style.gridRowEnd = "span " + rowSpan;
    });
}

function hide_tooltip_div() {
    const tooltip = document.querySelector("#tooltip") as HTMLSpanElement;
    tooltip.style.display = "none"
}

function setup_tool_tips() {
    const tooltip = document.querySelector("#tooltip") as HTMLDivElement;

    function onmm(e) {
        let parentContent = e.currentTarget;
        let newX = e.clientX + 10;
        let newY = e.clientY + 10;
        tooltip.style.top = newY + 'px'
        tooltip.style.left = newX + 'px'
        tooltip.style.display = "block";
        tooltip.innerText = parentContent.dataset.disc;
    }

    const images_that_need_tooltip = document.querySelectorAll(".gallery img") as NodeListOf<HTMLImageElement>;
    images_that_need_tooltip.forEach(content_object => {
        if (content_object.dataset.disc) {
            content_object.addEventListener("mousemove", onmm, false);
        }
        content_object.addEventListener("mouseleave", _e => {
            hide_tooltip_div();
        }, false);
    });
}