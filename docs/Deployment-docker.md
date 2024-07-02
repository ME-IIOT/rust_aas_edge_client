# Docker Deployment with Docker Compose

![plot](./images/docker-compose-structure.png)


## Sample Demo Deployment with Published Container Images

Folder "dist-docker-compose" contain docker-compose file and folders to be mounted as volumes to backend and frontend containers. 
The mount folders allow customizing frontend view and backend scripts, which is run by backend service to collect device data. 
These folders are showed in the figure above.

## Docker Deployment During Development

If the docker-compose at the root folder is used for deployment, the backend and frontend contaiers will be built directly from 
the source code in backend and frontend folders. No mount volumes is created for customization files (script, logos, etc). Instead, 
the files in the respective folders are used.